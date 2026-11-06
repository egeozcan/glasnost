---
id: S-0215
title: Behaviours (standing orders) and their processing
topic: simulation
revision: 132
sources-class: [decomp, measurement, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-06
measurements: [M-0023]
requests: [Q-0019, Q-0020, Q-0030, Q-0072, Q-0161, Q-0202, Q-0231, Q-0238, Q-0241, Q-0312, Q-0371, Q-0663, Q-0685, Q-0797, Q-0798, Q-0983, Q-1154, Q-1160, Q-1161, Q-1166, Q-2005, Q-2113, Q-2171, Q-2566, Q-2586, Q-2656]
supersedes: []
---

# S-0215: Behaviours (standing orders) and their processing

## Summary

Every infantry unit, vehicle, aircraft and building carries a *behaviour*, a standing order such as
guard, hunt, move or attack, and this spec covers how behaviours are held, queued, started,
forced, overridden and reverted, when and with which random delay an object processes its
behaviour, which resting behaviour it falls back to, and the per-behaviour properties that the
rules data can set. It also gives what the general-purpose behaviours and a gate's *open* behaviour
do at each processing, the approach step by which an attacking infantry unit or vehicle decides
whether to keep, renew or take a destination towards its target (claim 37, answering the behaviour
side of Q-0072; the approach position and failure outcomes are S-0307 claims 15 to 25), the area-guard threshold's engine default (the guard stray distance has
none in the original, and our engine's replacement is a documented departure), and pointers to the topics
that own the other behaviours (harvesting, structures, superweapons, team scripts). It also gives
the crush response, the one occasion on which a crusher goes for a crushable enemy that it does
not already hold as its target: a reaction to that enemy's hit (claim 39, answering Q-2005).

Under claim 32's non-SAM attack rule, a building
whose target leaves its weapon's range returns to *guard* at the first processing at which the firing check
answers *out of range*. While its rearm countdown runs, or while its animation-delayed shot is
pending, that check answers *rearming* instead, and the building keeps its target; an answer that
S-0301 ranks ahead of *rearming* still takes precedence (S-0301 claims 1, 9 and 14). Scenario
B16 case e is scoped to match: it judges the building by the firing check's recorded answer.

## Terms

This spec covers skirmish. Campaigns and network games are out of scope.

Terms from the glossary are used as defined there, in particular *tick*, *game minute*, *house*,
*human-controlled*, *lepton*, *cell*, *facing*, *foundation*, *object*, *infantry*, *vehicle*,
*aircraft*, *building*, *weapon*, *warhead*, *scatter*, *height above the surface*, *world height*,
*scenario random stream*, *rules layer*, *game-wide section*, *game start*, *recorder*, *measurement
setup*, *departure*, *importer warning*, *quirk switch*, *unspecified* and *round-toward-zero mode*.
*Sequence kind*, *interruptible*, *deploy kinds*, *frame count* and *idle action* are used as S-0107
defines them. *Turn* and *turn pass* are used as S-0203 (tick order) defines them. *Pending
suppression* is S-0207 claim 27's point suppression that has not yet been consumed: a one-application
skip of the next otherwise due nonempty point application. It is not the unchanged-destination
suppression of S-0502. *Late start* is the further per-tick start of an infantry unit or vehicle that
claim 11 defines. *Body facing*, *target facing* and *next point* are used as S-0206 (Terms)
and S-0207 define them: the body facing is the direction the vehicle's body points, the target
facing is the facing the body has been given and turns towards, and the next point is the next
position of the locomotor's path.

*Overload pulse* is the periodic self-damage of an unlimited controller, a mind controller whose
primary weapon gives unlimited control. S-0601 owns it (its Terms and claim 16).

Other specs cited: S-0001, S-0003, S-0006, S-0007, S-0009, S-0105, S-0107, S-0202 (random
streams), S-0203 (tick order), S-0204 (coordinates, distance and facing), S-0205 (number domains),
S-0206 (turn rates), S-0207 (ground vehicle movement), S-0208 (infantry movement), S-0209
(occupancy and crowding), S-0210 (crushing), S-0212 (pathfinding), S-0213 (human orders), S-0214
(order targeting rules), S-0301 (the firing cycle), S-0307 (weapon range test), S-0308 (weapon
choice), S-0309 (target acquisition), S-0312 (death and destruction), S-0402 (harvest cycle), S-0406 (building placement) and
S-0412 (skirmish setup rules), S-0417 (factory unloading, structures) and S-0501 (aircraft). All cited specs are approved; references also follow their current correction drafts.
This spec cites S-0412's Terms, its claims 3, 4 and 5 and its scenario K10 for the house skill
level and the difficulty index of each house at game start (claim 5 for the index of the neutral
and special houses).

**Requests.** This spec answers request Q-0020 in two of its parts: the engine default of the
area-guard threshold (claim 36 a) and the gate's *open* behaviour, which decides when a gate is
open (claim 35). Its third part, the guard stray distance, has no engine default in the original, and
by the project lead's delegated decision our engine uses the area-guard leash in its place, with an
importer warning (claim 36 b). It answers request Q-0019 in its part on behaviours:
which behaviours make a vehicle refuse every scatter request, and that the paralysed property
refuses only requests with a threat point (claims 17 and 19), with the vehicle's acceptance and
destination left to S-0207 claim 16. It answers request Q-0030, where within a gate's own turn its
door reaches rest, in claim 35 (d) and scenario B28 case g. Q-0072 is answered in part:
claim 37 gives the approach triggers, destination retention and renewal, and the building and
deployed-infantry exceptions. S-0307 claims 15 to 25 give covered searches and failure outcomes.
S-0207 claim 19 gives driving, sailing and hovering outcomes after coming in range. S-0208 claims 16 and 17 give the ordinary walker's stopping and destination outcomes.
The deploying-search gap remains open, so this is not a complete answer to Q-0072. Q-0161 is
answered in full by claim 37 a2: the per-tick approach follows processing and the per-tick
search, rechecks its conditions independently of a processing's approach, and can drop a
newly acquired target on that same tick. Q-0202 is answered by claim 13: refresh precedes
rather than follows the destination rows, with route-generated orders distinguished from
same-fall-back destination changes. Previously claim 13 and S-0309 claim 23 b already gave
refresh before the rest tables, but neither settled these collisions. Q-0231 is answered in full
by claim 26: the newly taken own-cell guard spot qualifies in that same processing, including
its search, targeting-timer restart and idle-action outcomes. The earlier version left that
qualification open; S-0309's area-search rule already supplies candidate eligibility, not a
one-processing deferral. Q-0241 is answered in full by claim 1: set-up placement runs the
building's grand opening after *guard* is queued, and the grand opening sets the ready mark.
The *guard* queued at set-up therefore becomes current at the building's first per-tick start
and is processed in that same turn. This holds alike for armed, unarmed, turreted,
non-turreted and set-up-upgraded buildings (question 2). S-0009 claim 40's "starts in the
*guard* behaviour" means *guard* queued at the end of set-up and current from the first
per-tick start, so every processing, the first included, is of *guard* (question 3).
Earlier revisions left the start open because they traced only the readiness rules after
placement. Q-0312 is answered
by claim 38 and F11–F12: the selected centre, no-base fallback, house records and exact
ranged/raw draw assignments. Q-0238 is answered jointly: S-0212 claim 23a already
supplies same-tick destination assignment, queue-only *area guard* and the final nearby
search; claim 38 supplies its centre. Keep Q-0238 open until the complementary revisions
are approved together.
Q-0371 question 3's behaviour consequence and subsequent processing are clarified by
claim 11a; S-0212 claim 23 owns the failure categories and route retry delay, and
S-0402 claims 3 and 13 own seeking and resumption. Q-0663 is answered by
claim 13: the owner-routes departure also uses the actor owner's house for both alliance
guards and the non-allied fallback, without a machine-local house. The same claim distinguishes
ordinary original skirmishes from our engine's AI-house route cases. S-0214's acting-house
and allied definitions already supply ordinary classification, not these extra route guards.
Q-0685 is answered by claim 35 d1–d3: opening and closing image positions, rounding,
endpoints, reversal and zero-time motion. Previously claim 35 d settled the rest tick and
S-0108 claim 4 settled image selection and clamping, but neither gave those positions.
Q-0797's behaviour side is answered by claims 22 and 29: neither ordinary vehicle
processing clears a goal merely for cell equality. S-0207 claim 26 owns cell-goal
cleanup and its tick timing; depot arrival exchanges remain S-0314 claims 17–18's.
Q-0798's gate side is answered by claim 32: immediate paired attempts take priority over
animation delay, a failed gate starts neither, and applicability uses upgrade position 1,
not installation history or the first nonempty position. S-0301 claims 9 and 14 answer
the empty-secondary effects and delayed-shot rules; S-0406 claim 12 supplies position/count
precedence, with weapon resolution owned by S-0308 claim 12. Previously claim 32 already
supplied the predicate and attempt order, but left these combinations unsettled.
Q-0983 is answered in full by claim 31: the immune deployer's own-cell secondary shot
is made during the deciding processing, without a fire-frame wait, and the own-cell
target is dropped in that processing after the shot. Previously the table already gave
the shot, target removal and sequence delay, but did not distinguish this immediate
shot from S-0107 claim 11's ordinary fire-frame shot. S-0525 claims 1 and 4 already
own radiation creation and top-up at detonation, not the decision to shoot.
Q-1166 question 1 is answered by claim 11: the late start also performs the fall-back to rest
(unmeasured, with the reason given there). Question 2 (self-healing against the berserk countdown)
is settled in S-0610 claim 6 for a berserk unit with no overload pulse: there the order has no
observable effect (backing `unmeasured`, confidence high). For an unlimited controller, whose overload
pulse is involved, the place of the countdown step is S-0601 claim 16's, as S-0610 claim 6 states.
This spec states no order of its own in either case. Q-1154 question 1 is answered by the Terms entry *enter-nearest instruction* and by claim 24: in our engine a unit of a neutral or special house holds the absorbing-building instruction, from the fate of S-0605 claim 16, and claim 26 row 3 applies to it. Question 2, whether a unit already waiting in line for the building is "already bound", is not answered here; it is listed under Open questions.

Q-1160 question 1 is answered by claim 11: a completed body turn is already complete at
that tick's first start, not only at its late start. S-0206 claim 1 already gives the
turn's tick-based completion; B47 now distinguishes same-tick processing from a late start.
Question 2 is answered by S-0207 claim 11, with proposed measurement D11 c:
attack during the pre-step turn clears both destinations, leaves no heading point and
cannot complete the cancelled step without a new locomotor destination. Cancellation
itself neither stops nor retargets the body turn. Subsequent aiming can change its target
facing independently; B47 retains its separate aiming and readiness qualifications.
S-0213 claim 21 owns the order boundary and later approach.
Q-1161 is answered in this spec's entirety by claim 11 and B48: an idle vehicle can start
an ordered move before arrival despite already having a locomotor destination. A retained
movement goal alone does not decide readiness. S-0207 claim 25 already owns when that
locomotor destination is assigned and retained; it is not itself the readiness predicate.

**Queueing wording.** Throughout this spec, "queues *b*" without further words means a queue
request (claim 7) *without* an immediate start. A rule that asks for the readiness-dependent
immediate start of claim 7 says "with an immediate start" (or "with an immediate start if
ready"), and a rule that makes the queued behaviour current regardless of readiness says "starts
it at once, whether or not it is ready to change".

**Houses in our engine.** The project lead decided on 2026-10-09 that AI houses (Terms) use the same
unit rules and the same order legality as human-controlled houses: no behaviour rule applies only
to computer-controlled owners, the recruitable property has no effect, and AI houses get no
exemption from the owner check or from the limit on pending orders. This is a rule of our engine,
not a claim about the original. Two further points are not part of the decision but follow from
it and from the project's scope. AI houses have no teams, because the original's team system is
out of scope. And an AI house has the house skill level (Terms) of a human-controlled house,
because the skill level is part of the unit rules. Throughout this spec:
- a rule given for a "human-controlled owner" applies in our engine to the units of every player
  house, human or AI,
- a rule, a table row or a condition given for a "computer-controlled owner" describes the
  original only. It is marked **Original only (note for AI designers)**, and our engine applies it
  to no player house in a normal game: no such player house meets the condition
  "computer-controlled", so for its units such a row never matches and such a condition never
  holds (the neutral and special houses, and test setups: below). What the original's computer player gets from
  the rule is recorded so that our AI may choose to reproduce it through ordinary orders,
- a condition on team membership never holds in our engine, because an AI house has no teams
  (Terms, *team member*),
- a rule gated by the house skill level, not by the owner, stays a rule of our engine. An AI house
  has the skill level of a human-controlled house: 0 in a skirmish (Terms, *house skill level*).
  The neutral and special houses have level 0 as well (S-0412, claim 3).

The ruling covers the player houses only. The two non-player houses that every skirmish creates,
the neutral house and the special house, are not AI houses: by the project lead's decision of
2026-10-10 they keep the original's rules for owners that are not human-controlled (glossary, *AI
house*, *Player houses only*). Where this spec says that a rule for computer-controlled owners
never applies, or applies to no house, in our engine, it speaks of the player houses; the units of
the neutral and special houses keep that rule. Likewise, where it says that a rule for
human-controlled owners holds in our engine for every house, for every owner or for AI houses
included, it means every player house. Claim 18 states this for retaliation. Rules that hold for
another reason are not affected. Campaign-specific rules are not specified here.

**Test-only computer-controlled player house.** By the project lead's decision of 2026-10-19,
our engine may mark a player house as computer-controlled in a test or arena setup for comparison
with the stock oracle. Such a house follows the original's rules for computer-controlled owners,
including this spec's original-only rows and conditions, exactly as neutral and special houses
do, but with the house skill level the corresponding oracle house had and with no teams. The
headless arena mode uses this setting for both houses, because both stock arena houses are
computer-controlled. It is unavailable in a normal game and is not given to our own AI. Throughout
this spec, statements that every player house follows human-controlled rules, or that an
original-only rule applies to no player house, exclude these explicitly marked test houses.
This includes retaliation outside *guard*, *area guard* and *patrol* (claim 18) and the
computer-controlled-owner condition of scatter case 2 (claim 19). It does not remove the
retaliate-property requirement: a hunter whose behaviour lacks that property cannot retaliate.

Each claim states its backing and confidence for the original's behaviour, including the parts
marked as the original's only. Which houses our engine applies a rule to is a design rule of our
engine, not a claim about the original, and needs no backing.

Terms used by this spec:

- **Computer-controlled, AI house, house skill level:** as the glossary defines them. This spec
  uses *computer-controlled* for the original and for the test-only setting below; an AI house
  follows the rules given for human-controlled owners (houses in our engine, above). For the house skill level, this spec owns
  the rules that the level gates (claims 14, 15 and 36), and S-0412 (claim 3) owns the level each
  house gets at game start: in a skirmish of the original the maximum skill level of the AI
  settings (below) for the house of every computer player, and 0 for every human-controlled house
  and for the neutral and special houses. In our engine an AI house in a skirmish keeps the level
  0 of a human-controlled house, and the neutral and special houses keep 0 as well.
  Campaign house-level assignments are outside this spec's scope.
- **Test-only computer-controlled player house:** a player house explicitly marked in a test or
  arena setup to use the original's computer-controlled-owner rules, with the oracle house's
  house skill level and no teams (houses in our engine, above). Headless arena mode marks both
  houses this way. This is not an AI house of our engine and is unavailable in a normal game.
- **Sequence frame count as a delay:** claims 22, 30 and 31 use the frame count of a unit's
  sequence entry (S-0107) unchanged as a number of ticks.
- **Actor:** an infantry, vehicle, aircraft or building object. Only actors have behaviours.
  "Mobile actor" means an infantry unit, vehicle or aircraft.
- **Behaviour:** one of the 32 standing orders in table T1 (Constants). Our names, in alphabetical
  order: *ambush*, *area guard*, *attack*, *attack move*, *capture*, *construction*, *eaten*,
  *enter*, *guard*, *harmless*, *harvest*, *hunt*, *missile*, *move*, *open*, *paradrop approach*,
  *paradrop overfly*, *patrol*, *queued move*, *repair*, *rescue*, *retreat*, *return*,
  *sabotage*, *selling*, *sleep*, *spy-plane approach*, *spy-plane overfly*, *sticky*, *stop*,
  *unload* and *wait*. In this spec a behaviour name in italics always means the behaviour, never
  the sequence kind of the same name in S-0107.
- **Current, queued and suspended behaviour:** three slots each actor holds. Each holds one
  behaviour or *none* (claim 1). The current behaviour is the one the actor carries out. The queued
  behaviour waits to become current. The suspended behaviour is the one an override put aside and a
  revert brings back (claim 10). An override is **in effect** while a behaviour is suspended
  (claim 10).
- **Processing:** one run of the current behaviour's rule for an actor (claims 2 to 5 and 22 to 33).
  A processing ends by choosing the **processing delay**: the number of ticks until the actor's next
  processing.
- **Processing timer:** the per-actor countdown that the processing delay sets (claim 2).
- **Behaviour start:** the moment a behaviour becomes current through a start-next or a force
  (claims 8 and 9). It restarts the processing timer so that the behaviour is processed at the next
  opportunity (claim 3), and it records the **start tick**, the tick of that start. An override or
  a revert is not a behaviour start and leaves the start tick as it was.
- **Progress count:** a per-actor integer with which some processing rules remember how far they
  have got within the current behaviour (for example a SAM building's aim-then-fire cycle, claim
  32). Every behaviour start sets it to 0; an override or a revert leaves it as it was. Its other
  values are internal to those rules, which state only their observable effects. Three rules look
  at the value itself: claim 13 asks whether a *patrol*'s count is still 0, claim 32 names the two
  stages (0 and 1) of a SAM building's *attack*, because a value outside them decides an observable
  delay, and claim 35 names the stages of a gate's *open*, which are the values 0 to 5 in the order
  of its table (*start* 0, *opening* 1, *waiting* 2, *closing-start* 3, *closing* 4, *finished* 5).
  The rules behave as if there were one progress count per actor, shared by all its behaviours,
  not one per behaviour. Among the rules of this spec, the sharing is observable only through
  claim 32, for a building with the SAM flag whose *attack* became current by an override while
  another behaviour's rule had left a value other than 0: a value of 1 puts that *attack* in its
  *aimed* stage without passing through *aiming*, and any value other than 0 and 1 gives claim
  32's out-of-stage rule. Of the behaviours this spec states, only a gate's *open* leaves such
  values (its stages 1 to 5, of which *opening* is the one that leaves 1); of the building
  behaviours of other topics, the processings of *repair* and
  *missile* leave some at certain stages, and those specs (structures and superweapons) own them.
- **Gate, door, passage state, close countdown:** a gate is as S-0209 (Terms) defines it; its door,
  passage state and close countdown are defined in claim 35, which owns when a gate is open or
  closed.
- **Ready mark:** a per-building yes/no state that makes a building ready to change (claim 11).
  Some building processings set it (claim 32), as do the construction state (S-0406 claim 17)
  and a building's grand opening (claim 1). A building without a turret also sets it when its body range advances onto its last stage, or
  on every non-advancing turn of a zero-delay range; a turreted building does so only while its
  behaviour is *construction* or *selling* (claim 11). A successful per-tick start clears it.
  Only its effect on when queued behaviours start is observable.
- **Grand opening:** S-0108's term (claim 11): the one-time step that brings a building into
  service. A building has at most one. This spec uses only one of its effects: it sets the
  ready mark. Claim 1 says when a building placed at set-up has it. S-0108, S-0402 and S-0009
  own its other effects.
- **Attack-move order, attack-move memory, eligible for attack-move:** S-0213 (human orders)
  owns the order and its initial memory; S-0214 (order targeting rules) owns eligibility.
  S-0309 claim 23 owns the memory's searches, refresh, removal and resumption. An attack-move order is the order that the
  attack-move gesture gives; the memory is what such an order leaves on a mobile actor (that its
  *move* or *attack* is an attack move, and where to); eligibility is a property of the actor's
  type (infantry or vehicle, with a weapon in its first slot, not marked as preventing attack-move; never
  a building or aircraft).
- **Distributed-fire candidates:** for an actor whose type has the distributed-fire flag, the list
  of candidate targets that its automatic search (glossary) collected, over which it spreads
  its fire. The target-acquisition specs own how the list is filled and emptied; this spec uses
  only whether it is empty (claim 24).
- **Behaviour properties:** the eight values per behaviour that the rules data can set: the
  *no-threat*, *paralysed*, *recruitable*, *retaliate*, *scatter* and *zombie* flags, the *rate* and
  the *anti-air rate* (claims 6 and 16 to 21; Import mapping). "The actor's properties" means the
  properties of its current behaviour.
- **Rate delay** *D*<sub>r</sub>: the processing delay derived from the current behaviour's rate
  (Formulas F1).
- **Draw *U*(*a*, *b*):** one ranged draw from the scenario random stream, uniform over the integers
  *a* to *b* inclusive. S-0202 (random streams) owns the generator and
  the reduction to a range; this spec states which draws happen and in which order.
- **Ready to change:** an actor's per-kind condition for letting its queued behaviour become current
  (claim 11).
- **Resting behaviour, falling back to rest:** when an actor has finished what its behaviour asked
  for, because a move arrived or a target died, it *falls back to rest*: it goes through the common
  effects of claim 13, then chooses a resting behaviour by claims 14 and 15 (aircraft: S-0501
  claim 18) and queues it. Aircraft have the early exits that claim 13 lists, which skip the
  common effects. The
  resting behaviours are mostly *guard* and *area guard*. The **placement fall-back** is the
  fall-back to rest that an actor makes at the moment it is placed on the map (created, or leaving a
  factory or a transport). Fall-backs made by orders, processings and damage are ordinary ones; a
  few made by other rules may count as placement fall-backs (Open questions).
- **Target, destination:** an actor's current target object (the thing it attacks, captures or
  enters) and its current destination (a cell or object it moves to). Either may be absent. How they
  are set by orders belongs to the order specs; how a target is chosen automatically belongs to the
  target-acquisition specs (combat topic).
- **Guard spot:** the cell or object an actor in *area guard* stays near (claim 26). An order to
  guard an object or a cell sets it; an actor that starts *area guard* without one takes its own
  cell.
- **Queued destinations:** the list of further destinations a mobile actor has been given in
  advance by orders. When it falls back to rest with no destination and a non-empty list, it takes
  the first entry as its destination (claims 14 and 15). The order specs own how the list is filled.
- **Approach destinations:** a second list of destinations, separate from the queued destinations,
  that a vehicle's own target approach sets up while the vehicle approaches a target (which parts
  of the approach fill it is not established; Open questions). Giving the vehicle a new destination
  empties it. Infantry and aircraft never have entries. This spec uses it in the common fall-back
  effects (claim 13) and in the approach step (claim 37, part e).
- **Approach step:** what an infantry unit or vehicle does about a target it holds, at the moments
  claim 37 lists: it checks whether the target is in range, and may keep, drop or renew its
  destination, take a new one, or drop the target (claim 37). Choosing the approach position is
  S-0307's (claim 15).
- **In the playfield:** a mark of each object that is set once the object has entered the usable
  area (glossary, *usable area*). An object that drives or flies in from outside it, for example
  one arriving at the map edge, does not have the mark until it has entered.
- **Scatter request, threat point, forced, move anywhere, tether link:** as S-0208 defines them
  (Terms). A rule of this spec that sends a scatter request always states its threat point (or
  that it has none) and both options.
- **Route:** a sequence of points that an actor has been ordered to visit one after the other
  (a route laid out by the player in advance). The order specs own routes and their points.
- **Contact:** the one pairing that an actor can hold with another object while the two carry out
  an exchange: a vehicle leaving the factory that built it, a harvester with the refinery it
  unloads at, a unit with the transport, repair bay or building it is entering, a vehicle with the
  bunker it occupies. Contact lasts until the exchange ends or one side breaks it off. When contact
  starts and ends belongs to the specs of those exchanges (production, economy, structures and the
  transport specs). A building can hold several contacts at once; the rules of this spec look only
  at a mobile actor's contact, which is one at most. An actor "in contact with nothing" holds no
  such pairing.
- **Bunker:** short for *tank bunker* (glossary), whose type has the bunker flag (Import
  mapping), and "in a bunker" means *bunkered* (glossary). Buildings that infantry occupy are the *occupiable
  buildings* below, never "bunkers".
- **Absorbing building:** a building whose type has the infantry-absorb flag or the unit-absorb
  flag (Import mapping): a building that takes in units as passengers, for example to draw power
  from them. Whether a given actor may enter a given absorbing building is decided by the building
  (structures specs).
- **Occupiable building:** a building that a given infantry actor may occupy (garrison) at that
  moment. S-0508 claim 1 owns that test, including the building type's can-be-occupied
  flag. An **owned occupiable building** of a house is a building of that house whose type has the
  can-be-occupied flag and a tech level of 0 or more (Import mapping); typically a building the
  house could build, rather than a neutral structure placed by a map.
- **Enter-nearest instruction:** one of three standing instructions that a mobile actor can hold,
  apart from its behaviour slots: *enter the nearest absorbing building*, *occupy the nearest owned
  occupiable building* and *occupy the nearest occupiable building*. The original's
  computer player gives them (team scripts, and the computer's decision on what to do with a unit
  it has just taken over by mind control). In our engine the only rule that gives one is the
  bio-reactor fate of a taken or released unit owned by a neutral or special house (S-0605 claim
  16, which gives *enter the nearest absorbing building* only); our AI gives none, and no other
  actor holds one, so claim 24's rule (and row 3 of claim 26's area-guard table) applies in our engine to
  the absorbing-building instruction only; the two occupy instructions never apply (note for AI
  designers). This spec says only how a
  holder's *guard* and *area guard* processings carry one out in the original and, for the absorbing-building
  instruction, in our engine (claim 24). An
  actor holds at most one effective instruction at a time for this purpose: when several are
  held, the first in the order just given is the one carried out.
- **Slave master, harvesting slave:** a slave master is a vehicle whose type controls a group of
  infantry harvesters, its *harvesting slaves*, which gather resources for it. The economy topic
  owns their rules: S-0402 (claim 14, the slave miner) states them, including the wake-up rule that
  says when the master **wakes** its slaves (sends them out to work). This spec only says where the
  slave rules interleave with behaviours.
- **Guard reset:** a rule that clears an actor's target, destination and guard spot and forces
  *guard* (claim 9).
- **Loaner:** an actor that the game marks as on loan: a transport that brings reinforcements onto
  the map and is not meant to stay. Which actors are marked belongs to the reinforcement and
  aircraft specs.
- **Targeting timer:** a per-actor countdown, separate from the processing timer, that paces the
  automatic search (claim 33).
- **Rearm time remaining:** the number of ticks until the actor's weapon may fire again after its
  last shot. The firing cycle owns the countdown (S-0301, claims 1 and 2); this spec uses
  only its remaining value.
- **Firing check:** the test that decides whether an actor can fire a given weapon at a given
  target now. Its answer is one of twelve categories, listed here in alphabetical order: *busy*,
  *can fire*, *cloaked*, *illegal*, *impossible*, *moving*, *must deploy*, *not facing*, *out of
  ammunition*, *out of range*, *rearming* and *still turning*. These are the answer names of
  S-0301 (claim 9), which owns the check, which situations give which answer and the precedence
  among them; S-0214 and the other combat specs give the answers of their own topics. This spec
  uses only the category: claim 32 says what a building in *attack* does with each one.
- **Firing turn, in the middle of firing:** an infantry unit's *firing turn* is the part of each
  of its updates, after both of its per-tick starts (claim 11), in which S-0107 claim 11's firing
  rules apply: the entry to the attack state, the shot at the fire frame and that claim's endings
  at the fire frame and with no target. An infantry unit is *in the middle of firing* while it is
  in S-0107 claim 11's attack state. Claim 11 here gives where that state begins and ends.
- **Armed:** as S-0214 (Terms) defines it: the actor has a weapon in the weapon slot it currently
  uses (its first slot, or, for a type that switches between weapons, the current one). A building
  that holds occupants and whose type lets them fire from inside also counts as armed. Every rule
  of this spec that says "armed" or "unarmed" (claims 11, 14, 15, 17, 21, 32 and 36 a) asks this
  one question.
- **Primary, secondary and deploy weapon:** an actor's *primary weapon* is the weapon in its first
  weapon slot and its *secondary weapon* the weapon in its second slot, in both cases the elite
  weapon of that slot instead when the actor is elite and its type gives one (the combat and
  veterancy specs own that choice). Its **deploy weapon** is its secondary weapon, or its primary
  weapon when its type has the spray-attack flag (Import mapping). So the deploy weapon of claim 30
  and the secondary weapon of claim 31 are the same weapon except for a type with the spray-attack
  flag. "No deploy weapon" means that slot holds no weapon.
- **Turreted building, voxel turret, turn-rate field:** a *turreted building* is a building whose
  type, or an upgrade installed on it, has the turret flag; its *turret facing* is the facing it
  aims with. It has a *voxel turret* when its own type's rules voxel-turret flag is set,
  the same flag used by S-0310 claim 4 and S-0313 claim 10. Art supplies no separate
  value, override or fallback. The *turn-rate field* is the type's imported turn rate, an integer (Import mapping); how it
  turns facings belongs to S-0206 (turn rates), and this spec uses it only in the bound of
  Formulas F8.
- **Crush response:** a crusher's reaction to a hit from a crushable enemy, by which it drives at
  that attacker to crush it (claim 39; S-0210 claim 11 owns who may make it and which attacker
  qualifies).
- **Opportunity target:** a target that an automatic search chose by itself and marked as such, as
  opposed to one given by an order. S-0309 (claim 24) owns which searches mark the targets they set
  (claim 33).
- **Charge target:** an own-house building selected by the automatic charging scan of S-0518
  claim 14. That claim owns eligibility, scan order and the charging mark. An actor aims at this
  building to power it up (claims 24 and 26).
- **Difficulty index:** the number 0, 1 or 2 that every house of the original has (S-0412, Terms),
  and the neutral and special houses have index 1 (S-0412, claim 5).
  The game-wide auto-deploy delay list gives one entry per index: its first entry for index 0,
  its second for index 1 and its third for index 2 (claim 31; Import mapping). In a skirmish the
  setup screen's hard computer entry gives index 0, its normal computer entry index 1 and its easy
  computer entry index 2, and every human-controlled house has index 1 (S-0412, claim 4). S-0412
  names the indexes after the difficulty rows they select, in the order easy, normal, difficult,
  which runs opposite to the setup screen's labels; this spec therefore writes the index and the
  setup-screen entry, not a difficulty name.
- **Team member:** an actor that belongs to a team of the original's computer-player script
  system. That system is out of scope for glasnost, so this spec only notes where membership
  changes a rule of the original. An AI house of our engine has no teams (houses in our engine),
  so a rule's exception or condition for team members never applies to its units.
- **2-D distance, 3-D distance:** the distance between two objects' reference points in leptons,
  from the horizontal coordinates only (2-D) or from all three (3-D). The original takes it with the
  *table square root* (the approximate root that approved S-0105, F1, also uses), truncated toward
  zero to an integer, not with the exact root (Formulas F4). This spec states that root itself.
- **Type words.** Words that describe an actor by its type, such as *engineer*, *occupier*,
  *assaulter*, *infiltrator* ("its type infiltrates"), *vehicle thief*, *charge planter* ("has
  explosive charges"), *harvester*, *weeder*, *hover-attack type*, *deployer* ("a type that
  deploys"), "deploys to fire", "immune to radiation", the *undeploy delay*, *guard range* and
  *sight*, and the building words *vehicle-factory building*, *unit repair building*, *SAM flag*,
  *electromagnetic-pulse cannon*, "accepts charges", "invisible", "needs power" and the
  *stupid-guard-mode*, *stupid-hunt*, *close-range*, *distributed-fire* and *opportunity-fire*
  flags, each mean the imported type field of that role in the Import mapping. What those fields
  do outside this spec belongs to the specs that own them.
- **Fraidycat:** an infantry type whose type has the fraidycat flag, as S-0107 (Terms) defines
  it; S-0208 (claim 11) uses the same word in its acceptance rule for scatter requests. This spec
  uses the word only in scenario B29, to keep such types out of that test.
- **Short of power:** a house is short of power when its power output is less than the power its
  buildings drain (the ratio of output to drain is below 1). The power rules belong to the
  structures topic; this spec uses only whether a house is short of power (claim 32).
- **Construction yard:** a building whose type is in the computer player's list of
  construction-yard types (Import mapping). "A vehicle that deploys into a construction yard"
  is a vehicle whose deploys-into type is in that list. Base building belongs to the structures
  and production topics.
- **Automatic base building:** a per-house setting of the original's computer player under which
  it builds its own base. Only rules marked as the original's only (claims 24 and 27) look at it; the
  computer player's rules own it, including when it is switched on (in a skirmish that can be
  from the start of the game, depending on the house's skill level), so this spec makes no claim
  about it; scenario B11, case f, reads it with the recorder.
- **Mind control:** an actor's capture of another actor, which then changes owner to the
  capturer's house. S-0601 claims 6 and 9 own takeover and release. Takeover normally gives
  the captured actor a guard reset (claim 9); a vehicle that deploys in place and is currently
  in *unload* is excepted as S-0601 claim 6 states. Release is not an explicit guard reset,
  but its ownership change clears target and destination and normally requests *guard* with
  a readiness-dependent immediate start (claim 7), rather than preserving the last order
  (S-0601 claim 9).
- **Being erased:** as the glossary defines it. The combat topic owns erasing weapons.
- **Cloak, sensor:** an object that *starts to cloak* begins to turn invisible to other houses,
  and a *sensor* of a house detects cloaked objects in the cells it covers. S-0516 claims 1–9
  own the cloak states and when an object starts to cloak; S-0516 claims 21–23 own sensor
  coverage. This spec uses them only as a revert cause
  and its exception (claim 10).
- **Chrono move, leaving stage, phase-in:** a chrono move teleports an actor to another place, by a
  superweapon or by its own teleporting locomotor. A superweapon move's **leaving stage** runs, for
  each member of its warp group that the move moves, from that member's first update after launch
  until departure, including the update at which its 60-tick wait expires (S-0513 claim 39). A
  teleporting locomotor's own jump has no leaving stage (S-0207 claim 20 d). **Phase-in** is
  reappearance at the new place. The superweapon topic owns its move's stages and lengths. S-0207
  claim 20 f and F9 own the locomotor jump's phase-in length and harvester exception.
- **Tunnel:** a pair of linked tunnel entrances on a map through which infantry and vehicles
  travel out of sight to the other entrance. The movement topic owns it.
- **Radiation level:** S-0525 claim 7 defines a field's current integer *field level*.
  Claim 31 tests only a field centred on the unit's cell, not the cell's radiation exposure.
  A weapon's *radiation level field* is the level it leaves (Import mapping).
- **Veterancy rank, veteran, elite:** an actor's *veterancy rank* is the experience rank that the
  veterancy specs give it; *veteran* and *elite* name ranks of those specs. This spec uses the
  words only as pointers: for the elite weapon (Terms, *primary, secondary and deploy weapon*), for
  the veterancy abilities below and in scenario B4. The veterancy specs own the ranks and how an
  actor reaches them.
- **Veterancy ability:** a capability that an actor's type grants it once it reaches a given
  veterancy rank, listed per type in the veterancy ability lists. This spec uses the guard-area,
  explosive-charges and scatter abilities; the veterancy specs own the lists.
- **AI settings, area-guard threshold:** the *AI settings* are the game-wide rules values that
  set the computer player's intelligence levels; among them the *maximum skill level* and the
  *area-guard threshold*, an integer house skill level. A house whose skill level is below the threshold
  rests its units in *guard* rather than *area guard* in the cases of claims 14 and 15 (Import
  mapping).
- **Deactivated, reactivated:** an actor is *deactivated* when another topic's rule switches
  it off, for example a vehicle disabled by loss of its control centre, and *reactivated* when
  that rule switches it on again. An electromagnetic pulse is a separate state and does not
  by itself deactivate a vehicle (S-0405 claim 13). At the enabling transition, every on-map
  vehicle of the affected powered-unit type receives the guard reset, whether or not it was
  deactivated (S-0405 claims 13 and 20).
- **Open-topped transport:** a transport (an actor that carries passengers) whose passengers can
  fire from inside it. The transport specs own both.
- **Unload or deploy cycle:** the cycle named by the readiness and restart conditions in
  claims 11, 22 and 34. No rule starts it for a vehicle, so for vehicles it is always finished
  and those restart conditions never take effect (S-0517 claim 15). It is distinct from the
  in-place vehicle deployment of S-0517 claims 3 to 8.

## Behaviour

### Holding and processing behaviours

1. **Three slots per actor.** Every actor has a current, a queued and a suspended behaviour. A newly
   created actor has *none* in all three. Other objects (projectiles, animations, overlays) have no
   behaviour.

   As a general rule, when an actor has no current behaviour, rules that ask for its behaviour
   see the queued one. This does not start that behaviour or fill the current slot: processing
   still follows claim 4 while the current slot is *none*. Aircraft have one exception: the
   scatter gate of S-0501 claim 49 never reads the queued behaviour, so an aircraft with no
   current behaviour falls back to rest on a scatter request whatever is queued.

   A building's placement fall-back queues *construction*, rather than choosing a resting
   behaviour, and requests the construction body (S-0406 claim 17). At game set-up it instead
   receives the idle body and *guard* queued. Creation starts with the ready mark clear.
   The set-up choice of idle body and queued *guard* does not set the mark, and placement's
   readiness test, which comes right after that choice, finds the building not ready, so
   *guard* stays queued for now. Later in the same placement, the building's grand opening
   (Terms) runs, and it sets the ready mark. This is a fixed rule of set-up placement for
   a building whose owner is not human-controlled, which in a skirmish is every building
   the map places (S-0009 claim 27).

   So when set-up ends, the building has the idle body, current *none*, *guard* queued and
   the ready mark set. At the building's first per-tick start (claim 11; its first turn, the
   tick of which S-0203 gives) *guard* becomes current and the mark is cleared, and *guard*
   is processed in that same turn (claim 3 a). From there an armed building's search and
   first-shot eligibility follow claims 32 and 33, and an armed building's *guard*
   processing sets the mark again (claim 32). A building armed only by an installed
   upgrade can acquire a target and start *attack* yet never fire: claim 32's upgrade
   weapon gate is independent of this bootstrap.

   None of this depends on the building's weapon or turret. Armed or unarmed, turreted or
   not, the building starts *guard* at the same per-tick start. An upgrade that the map
   installs on the building (S-0009 claim 40) changes neither the host's slots nor its
   ready mark. The upgrade object is never placed as a building of its own and has no grand
   opening. A host whose turret comes from such an upgrade therefore starts like any other.
   This is the meaning of S-0009 claim 40's “starts in the *guard* behaviour”: *guard* is
   queued at the end of set-up and current from the building's first per-tick start, so
   every processing of the building, its first included, is a processing of *guard*.

   Only the ready-mark effect of the grand opening belongs to this spec. Which of its other
   effects a set-up building shows is for S-0009 claim 40, S-0108 claim 11 and S-0402
   claim 18 to say. For a set-up building whose owner is human-controlled, which a skirmish
   never places, the original would run the grand opening before *guard* is queued, so
   placement's readiness test would already make *guard* current.
   - Backing: `measurable` (scenario B36 for the set-up grand opening's ready mark,
     end-of-set-up slots, first per-tick start and same-turn processing, including the
     armed, unarmed, non-turreted and map-installed-upgrade comparisons; scenario B1
     for the other slot rules; S-0406 PL11 for construction entry and the queued-behaviour
     query; scenario B35 for the body and queue at set-up). The human-owner set-up case is `unmeasured`: skirmish drops
     that owner's map building entries, and construction or capture cannot isolate set-up
     placement. The aircraft exception is `unmeasured`, as S-0501 claim 49 rates it: no setup
     is known that sends an aircraft a scatter request before its first turn.
   - Confidence: high (the slot rules and the set-up and placement rules); medium (the aircraft
     exception of S-0501 claim 49)

2. **When an actor processes its behaviour.** On each tick on which an actor's own update runs (its
   turn in the turn pass of S-0203, tick order, which says which objects
   take turns, in which order, and when an object loses a turn), the actor
   processes its current behaviour if, and only if, all of these hold, subject in our engine
   to the superweapon-warp design exclusion below:
   - the actor is alive and its health is above 0,
   - its processing timer has expired,
   - it is not an infantry unit or vehicle travelling through a tunnel,
   - it is not an infantry unit or vehicle that is being erased, or that is in the leaving stage of a
     chrono move (Terms), nor a vehicle that is phasing in with a temporal beam attached while it
     is not being erased, as a superweapon warp member is after its departure (S-0513 claim 39a).
     With S-0513 `quirks.phase_in_beam_spares_infantry` off, an infantry unit in that state is
     excluded as well. Such an actor also drops its target on each such tick, and drops its
     destination on each such tick, except that a vehicle under a pending point suppression
     (S-0207 claim 27) keeps its destination, whether it is being erased or in the leaving stage.
     The target is dropped in every case, and the exclusion from processing is unchanged. These
     sentences apply to the phasing-in cases of this bullet too,
   - it is not a landed aircraft being erased, nor a landed aircraft phasing in with a temporal
     beam attached while it is not being erased (S-0513 claim 39a). Such an aircraft drops its
     target and destination
     and skips ordinary movement, turning, firing and behaviour processing for that turn
     (S-0514 claim 17). This does not exclude its erasure update or presentation effects. These
     sentences apply to the phasing-in case of this bullet too.

   On a tick on which the tunnel, erasing or leaving-stage exclusion holds (the last three bullets
   above, the phasing-in cases added to them included), no per-tick start of claim 11 takes effect
   (the first start, and for infantry and vehicles also the late start defined there), and neither
   does the berserk countdown (S-0610 claim 5), which does not otherwise depend on the processing
   timer being due. Subject also to the superweapon-warp design exclusion below, on every other tick
   on which the actor does not process, the per-tick starts still take effect: for example on a tick
   on which only the timer has not expired (claim 3a, scenario B3, case k). A teleporting
   locomotor's own phase-in does not by itself stop processing: the actor processes under the
   ordinary eligibility rules, the exclusions above for an actor phasing in with a temporal beam
   attached included.

   **Superweapon warp, engine design.** S-0513 claim 51 excludes ordinary behaviour processing
   and the per-tick behaviour start of claim 11 for vehicles, teleporter infantry and landed
   aircraft from the first pending update through arrival effects, including departure, every
   off-map landing retry and placement, whether successful or failed. Neither ordinary
   processing nor a per-tick start occurs before or after warp work on those turns, even for
   a survivor of the arrival effects. During arrival hold, the hold-end test precedes ordinary
   behaviour eligibility: processing and per-tick starts regain their usual eligibility both
   while the hold continues and on its expiry turn. That usual eligibility includes the
   exclusions above: while it phases in, a vehicle or landed aircraft member with a temporal beam
   attached, being erased or not, neither processes nor takes a per-tick start, and neither does
   an infantry member while it is being erased or, with S-0513
   `quirks.phase_in_beam_spares_infantry` off, while a beam is attached to it (S-0513 claims 39a
   and 51). This is not permission to ignore
   another exclusion, nor does it suppress the target searches or sensor update intrinsic to
   the warp (S-0513 claim 39). It is an engine design rule, not confirmed original behaviour,
   and has no quirk switch. S-0513 claims 48 and 52 retain the original-side uncertainty about
   processing and per-tick-start continuation on pending, departure, retry, placement,
   arrival-effects and hold turns.

   A processing on tick *t* that chooses the delay *d* sets the timer so that it has expired at
   any later turn of the actor on a tick *t*′ with *t*′ − *t* ≥ *d*. The actor processes again on
   the first tick *t*′ > *t* with *t*′ − *t* ≥ *d* on which its update runs, so a delay of 1 or
   less (0 and negative delays included) means "again on the next tick". A turn processes at most
   once. An actor normally has one turn per tick; when S-0203 (claim 11) gives it a second turn
   in the same tick, that second turn processes again only if the delay chosen in its first turn
   was 0 or less (or a behaviour start, claim 3, has set the timer to "expired now" since).
   For an ordinary vehicle turn, due behaviour processing sees the destination state before
   that turn's locomotion changes it. Final-track completion itself neither expires nor
   restarts the processing timer and causes no extra processing. Arrival exchanges and
   behaviour starts remain separate effects governed by their own rules. In particular,
   for the uninterrupted field arrival of S-0402 claim 3b, let *T* be the final track's
   completion tick. A due seeking processing in tick *T* + 1 still sees the order-level
   cell destination, remains seeking and chooses the ordinary standard delay of *harvest*
   (claim 6), including its one delay draw. Locomotion removes that destination later in
   the turn (S-0207 claim 26). S-0402 claim 3b gives the resulting loading-start timing.

   Due behaviour processing precedes an attached parasite's same-turn delivery or grappling
   event and its hold renewal (S-0614 claim 18). A hit mark or a newly renewed hold from that
   event does not undo the behaviour processing or shots it already made, including an
   aircraft's attack-run shots. This does not place all firing before the parasite:
   S-0614 claim 18 distinguishes the later separate firing of infantry and vehicles.
   - Backing: `measurable` (scenarios B1 and B18; S-0402 claim 3b, scenario H27 for
     processing before locomotion and the unchanged arrival timer; S-0514 E11 part a for landed-aircraft erasure,
     S-0614 scenario P12 for parasite ordering). The warp design contract is `measurable`
     in our engine (S-0513 scenario SW23 part f); the original's unresolved continuation
     remains `unmeasured` as S-0513 claim 48 states (SW23 parts a to e, incomplete per-kind
     eligibility evidence). The second-turn case is `unmeasured`: S-0203
     marks the second turn itself as not observable by a recorder. No original-side rule
     is asserted here for an erasure overlapping a chrono arrival; S-0513 and S-0514
     own those effects, and no isolated fixture is established here. The berserk countdown
     on the ticks that the tunnel, erasing and leaving-stage exclusions hold takes S-0610
     claim 5's backing: `measurable` for the erasing and leaving-stage ticks (S-0610 scenario P9
     parts d and e), `unmeasured` for the tunnel and landed-aircraft ticks (S-0610 claim 5 gives
     the reason: no setup is known). The exclusion of the late start of claim 11 on those ticks is
     `unmeasured`: B46 uses an expiry tick on which none of those exclusions holds, and no setup
     is known that puts the expiry on an excluded tick together with a ready, queued unit.
     The phasing-in cases with a temporal beam attached are `unmeasured` for an actor that is
     not being erased (S-0513 claim 39a: no part there holds an infantry member, SW12 part l has
     a tank only, and its hold of one update is too short to tell a turn cut short from an
     ordinary one), and take this claim's backing for an actor being erased. The off setting of
     S-0513 `quirks.phase_in_beam_spares_infantry` is `design`.
   - Confidence: high (alive, health, timer, parasite ordering and the engine design contract),
     medium (the tunnel, erasing and leaving-stage exclusions, including landed-aircraft erasure,
     and locomotor phase-in not excluding processing, and the composed field-arrival timing); low
     (the second-turn case and
     matching the original's unresolved superweapon-warp continuation); the berserk countdown on
     the tunnel, erasing and leaving-stage ticks medium, and on the landed-aircraft tick low (S-0610
     claim 5); the exclusion of the late start medium; the phasing-in cases with a temporal beam
     attached medium for an actor that is not being erased, and as the erasing exclusion for an
     actor being erased
   - Pending-suppression exception (the vehicle clause of the erasing and leaving-stage bullet):
     Backing: `unmeasured`. S-0207 claim 27 finds its reachability not established (its Reachability
     paragraph and its table row for a suppressed vehicle in any state, including one being erased),
     and no scenario here creates the state. Confidence: medium (the rule as S-0207 claim 27 states it).

3. **A behaviour start is processed at once, unless the processing that caused it chooses later.**
   A behaviour start (start-next, claim 8, or force, claim 9) sets the processing timer to "expired
   now". So:
   - (a) a start on tick *t* that precedes the actor's processing for tick *t* (for example one
     caused by another actor's update earlier in the tick) leads to a processing of the new
     behaviour on tick *t*. In particular, a queued behaviour that an infantry unit, vehicle or
     building makes current at its first per-tick start (claim 11; for a building, its first per-tick
     start of the tick) is processed on that same tick,
   - (b) a start that happens after the actor's processing for tick *t* (for example one caused by
     another actor's update later in the tick, a building's second per-tick start, an infantry unit's
     or vehicle's late start, or an aircraft's end-of-update start, claim 11) is
     processed on tick *t* + 1 (or in a second turn of the same tick, claim 2),
   - (c) a start that a processing itself causes is overridden by the delay that processing then
     chooses: the new behaviour is first processed when that delay runs out. Each claim below that
     starts a behaviour from within a processing says which delay it chooses.

   An override or a revert (claim 10) does not touch the processing timer: the behaviour it makes
   current is first processed when the timer set by the last processing runs out.
   - Backing: `measurable` (scenario B3, cases a and k, for the reset by a start; scenario B7,
     case b, for the override leaving the timer alone)
   - Confidence: high (timer resets); medium (that no other rule touches the timer)

4. **Behaviours without a processing rule.** For the behaviour and actor-kind pairs marked "—" in
   table T1, a processing does nothing and chooses a delay of exactly 450 ticks, with no draw. This
   holds for *sleep*, *harmless*, *ambush*, *return*, *stop* and *wait* for every actor kind. An
   actor whose current behaviour is *none* processes in the same way. This also covers a
   building's *move*: it does not move the building, request packing or clear its guard spot.
   S-0402 claim 27a gives a reachable case: a field-seeking slave-master building requests
   *move* without an immediate start, and readiness can permit it to become current under
   claim 11. The group's subsequent return to rest does not cancel that *move* or restore
   *guard*; while *move* remains current, the previous behaviour's processing no longer runs.
   - Backing: `measurable` (scenario B1, for the six behaviours and the infantry, vehicle and
     aircraft kinds; S-0402 claim 27a and scenario H25's site-offer variants for building
     *move*). The six listed behaviours for buildings remain `unmeasured`: a map's building
     entries carry no behaviour, and no order, trigger or other input is known that puts a
     building into one of those six behaviours. The *none* case is `measurable` for fresh
     paratroopers (S-0522 claim 17 and scenario B44): placement queues a resting behaviour,
     but falling prevents its start, leaving *none* current for due processing. This does
     not change claim 11's fall-back for infantry with neither current nor queued behaviour.
   - Confidence: high

5. **Behaviours that share another behaviour's processing.** For every actor kind:
   - *sticky* is processed exactly as *guard* (claim 25 says what makes it differ),
   - *sabotage* is processed exactly as *capture* (claim 28),
   - *queued move* and *attack move* are processed exactly as *sleep*: nothing happens and the delay
     is 450 ticks (`quirks.queued_move_sleeps`).

   No rule of the original's program makes *queued move* current; only data that names it can (for
   example an object that a map places with that behaviour). *Attack move* becomes current in the
   same way, and also through the attack-move orders that S-0213 (claim 6) turns into *attack move*
   itself (claim 12). Ordinary player input can create attack-move orders, but the eligible
   target-only and destination-only orders produce *attack* and *move*, respectively, rather
   than the standing behaviour *attack move*. As far as studied, ordinary player input does not
   create an attack-move order whose translation produces the standing behaviour *attack move*.
   Thus, within the ordinary-input cases studied, data is the only established way to put an
   actor into *attack move*.

   Falling does not suppress an ordinary infantry unit's or vehicle's due behaviour processing.
   The fall step and any landing hit precede that processing, which may change orders under
   its usual rules. While still falling, the unit cannot advance its ordinary locomotion or
   fire; a surviving landing removes those falling-specific refusals in that same turn
   (S-0511 claim 10).

   A shared processing still reads the properties of the behaviour that is current: an actor in
   *sticky* uses the rate (and, for a building, the anti-air rate) of *sticky*, not of *guard*, and
   an actor in *sabotage* uses the rate of *sabotage* (claim 6).
   - Backing: `measurable` (scenario B14 for *queued move* and *attack move*; scenario B22 for
     *sticky* and *sabotage* and for the rate each reads; S-0511 scenario B3's turn-order
     variant for due processing during falling and the surviving-landing boundary). For buildings the shared processing of
     *sticky* and its anti-air rate are `unmeasured`: a map's building entries carry no behaviour
     and no known input puts a building into *sticky*.
   - Confidence: high (the shared processing); medium (that no other rule of the program makes
     either behaviour current)

6. **The rate delay and the standard delay.** Most processings end with the **standard delay**
   *D* = *D*<sub>r</sub> + *U*(0, 2): the rate delay of Formulas F1, computed from the current
   behaviour's rate in game minutes at 900 ticks per game minute and truncated toward zero, plus one
   draw. Table T2 lists, for the general-purpose processings of mobile actors and the infantry
   rules of claims 23 and 27, which delay each outcome chooses and how many draws it takes; the
   claims give the other kind-specific outcomes. A delay
   takes a draw only where its formula shows one (*U*(*a*, *b*)). Every other delay of this spec,
   whether a fixed number of ticks (1, 2, 100, 450), a frame count, the undeploy delay, the rearm
   time remaining, 0 or *D*<sub>r</sub> alone, takes no draw for the delay itself; a target search
   that the same processing runs still takes its own draws (T2). Because the rate is held in single
   precision (S-0003, claim 6) and the delay is truncated, a rate that is written as a short
   decimal can give one tick less than the decimal times 900: a rate written as 0.06 gives
   *D*<sub>r</sub> = 53, not 54 (Formulas F1). The shortfall comes from the single-precision
   reading alone: the product of a single-precision rate with 900 is exact (S-0205, claim 9), so
   it does not depend on the rounding mode. Only a rate written with a percent sign is a
   double-precision product, and its product with 900 depends on `quirks.chop_rounding` (S-0205).

   The rate is read from the behaviour that is current at the moment the delay is chosen. When a
   processing has itself made another behaviour current (claim 3c) and then chooses a delay from a
   rate, it uses the rate of the behaviour it has just made current. Among the processings that
   this spec describes in full (claims 22 to 28 and 30 to 32), those that can do this are:
   - *hunt*, whose general with-target outcome can start *capture* or *sabotage* (claim 27),
   - *guard* and *area guard* carrying out one of the two occupy instructions (original only),
     which can start *capture* (claims 24 and 26, row 3),
   - an aircraft in claim 28's second row, whose fall-back to rest, the first one and the second one
     at a *scatter off a building* request, makes current the behaviour that S-0501 table T2
     chooses, for instance *enter* (claim 28, S-0501 claims 18 and 49),
   - a processing whose fall-back to rest issues a route point by claim 13's route row and then
     chooses the standard delay, such as *attack* with no target (claim 23): the queue of *wait*
     with an immediate start before the issue (S-0213 claim 16c) makes *wait* current for a holder
     that is ready to change, so that delay uses the rate of *wait* (medium, `measurable` by
     scenario B15 part g).

   A behaviour that a processing only queues, without making it current, leaves the rate as it was.
   The starts of *hunt*, *guard*, *area guard* and the route row's *wait* above are
   readiness-dependent: when the actor is not ready to change, the new behaviour stays queued and
   the rate is that of the behaviour still current. Which behaviour an aircraft's fall-back leaves
   current follows S-0501 claim 18's end states. The infantry rules of claims 23 and 27 that end
   with 1 tick take no rate at all.

   **Observable random-stream result.** In a processing that takes a delay draw, its delay
   receives the stream value after the values assigned to every other draw belonging to that
   processing. Those other draws include a target search (claims 26 and 27, including the
   search's *U*(0, 2) for the targeting timer, claim 33), and rules such as approaching a target,
   an idle action or a scatter request, whose draw counts belong to their own specs. Thus, in an
   *area guard* processing whose search runs, the targeting-timer value corresponds to the
   earlier *U*(0, 2) assignment and the delay to the subsequent *U*(1, 5) assignment. The required
   results are these timer values and the resulting random-stream state, not an internal
   execution order.
   - Backing: `measurable` (scenario B2 for the rate delay and the standard delay, its third run
     for a rate written with a percent sign under the original's rounding; scenarios B11
     case e and B21 for the rate of a behaviour the processing has started, and scenario B22 for the rate of the
     current behaviour under a shared processing; scenarios B11 case b and B21 for fixed
     delays taking no draw; scenario B27, case d, for the timer and delay draw assignments
     in *area guard*; scenario B30, case a, for a rate whose product with 900 reaches 2⁶³,
     which F1 turns into 0). The not-a-number case of F1 and F9 is `unmeasured`: S-0003's decimal
     form never yields a not-a-number, so imported data cannot reach it (F1). The delay draw
     following a targetless vehicle's *hunt* search is `measurable` (B45 a). Assignment after
     approaching, infantry idle actions and scatter requests remains `unmeasured`: their
     draw counts are not fully specified by their owning specs, so total stream advance
     alone cannot identify the draw assigned to each rule. The aircraft item of the list above is
     `unmeasured`, as claim 28 and S-0501 claim 49 rate an aircraft's answer: no scenario
     records the rate taken after an aircraft's fall-back has changed its current behaviour.
   - Confidence: high, except the route-row item of the list above (medium, B15 part g) and the
     aircraft item (medium, as S-0501 claim 49 rates an aircraft's answer, and low for the outcome
     that claim 28 rates low)

### Changing behaviours

7. **Queueing a behaviour.** A request to queue behaviour *b* (with or without an immediate start)
   has the effect of the first row that matches:

   | Situation | Queued behaviour afterwards | Immediate start, when asked for |
   |---|---|---|
   | **aircraft:** the actor is not an airstrike aircraft, its current behaviour is *retreat*, either paradrop behaviour or either spy-plane behaviour, and *b* is none of those five (S-0501 claim 34) | unchanged | none: the request is ignored entirely |
   | the current behaviour is *selling*, or it is *wait* and *b* is *guard* | unchanged | none: the request is ignored entirely |
   | *b* is *none* | unchanged | a start-next (claim 8) if the actor is ready to change (claim 11) |
   | *b* is the current behaviour, and nothing or *b* itself is queued | unchanged | as above |
   | any other case (*b* differs from the current behaviour, or a behaviour other than *b* is queued) | *b*, replacing any queued one | as above |

   Queueing the current behaviour while nothing is queued changes nothing; queueing the current
   behaviour while another one is queued replaces the queued one with it, so the next start
   restarts the current behaviour.

   Some rules instead queue a behaviour and then make it current with a start-next that does not
   look at readiness. The claims that describe such a rule say that it starts the behaviour "at
   once, whether or not the actor is ready to change"; "with an immediate start" always means the
   readiness-dependent start of this claim; and a rule that says only "queues" asks for no
   immediate start (Terms, queueing wording).
   - Backing: `measurable` (scenario B3; case l for the replacement of a queued behaviour, case i
     for the refusal under *wait*). The aircraft row has S-0501 claim 34's backing: `measurable`
     by S-0501 scenario AC29 part h for a visiting carrier slot aircraft in *retreat*, and
     `unmeasured` for the other four behaviours, which no scenario queues anything for, and for
     the airstrike-aircraft exception.
   - Confidence: high (the other rows); medium (the aircraft row, as S-0501 claim 34 rates it)

8. **Start-next.** If a behaviour is queued, it becomes current, the queue becomes empty, and the
   behaviour start of claim 3 happens: the timer expires now, the start tick becomes the current
   tick, and the progress count becomes 0. The suspended behaviour is not touched
   (`quirks.stale_suspended_revert`). If nothing is queued, nothing happens.
   - Backing: `measurable` (scenario B3)
   - Confidence: high

9. **Force.** A forced behaviour *b* becomes current at once, whether or not the actor is ready to
   change, unless the current behaviour is *wait* and *b* is *guard* (then nothing happens). The
   queue becomes empty and the behaviour start of claim 3 happens. Unlike queueing and overriding,
   force is not refused while the current behaviour is *selling*: a forced behaviour replaces
   *selling* (no known input forces a behaviour on a selling building). The suspended
   behaviour is not touched (`quirks.stale_suspended_revert`).

   Rules that force a behaviour include the guard reset (Terms), which the original applies to an
   actor taken over by mind control (with S-0601 claim 6's in-place-deployer exception), to an actor that is deactivated or reactivated, to an actor
   that enters an open-topped transport, to an actor whose locomotor is replaced by another,
   and to harvesting slaves through the slave rules (economy topic); and the processings of claims 23 and 27 (their original-only rules), which force *capture*, and of claim 30, which forces *guard*. No
   player's order is known to force a behaviour on the actor it is given to (claim 12); the stop
   order given to a slave master gives its harvesting slaves a guard reset. S-0207 claim 28's
   temporary driving and the restorations of S-0207 claim 27 and of claim 13 here are not
   replacements of the locomotor in this sense: they leave the behaviour as it is (S-0207 scenario
   D34 part c observes both).
   - Backing: `measurable` (scenario B3, cases f and i, for the force and the mind-control guard
     reset; case g for the stop order's guard reset of a slave master's slaves; also
     B45 f for disabling and enabling a control-centre-dependent vehicle; S-0405 claim 20,
     scenario W8 part l for enabling resetting an already enabled vehicle; S-0607 L2 for
     magnetic seizure's guard reset when its locomotor is replaced). Entering an open-topped
     transport and additional slave-rule reset sites remain `unmeasured` here: this spec
     supplies no isolated fixture for those sites, and the generic force test alone does
     not establish that each site invokes it.
   - Confidence: high (the rule); medium (the list of rules that force)

10. **Override and revert.** An override puts a temporary behaviour *b* in place with a new target
    and destination, keeping enough to come back:
    - in every case the actor's target and (for mobile actors) destination are put aside and
      replaced by the override's,
    - the slots do not change if the current behaviour is *selling*, or the current behaviour is
      *wait* and *b* is *guard*; the target and destination are replaced all the same
      (`quirks.refused_override_retargets`),
    - otherwise the suspended behaviour becomes the queued behaviour if one is queued, else the
      current behaviour (`quirks.override_suspends_queued`); *b* becomes current; the queued
      behaviour stays queued.

    A revert does nothing unless a behaviour is suspended. If one is, it becomes current, the
    suspended slot becomes empty, and the target and destination put aside by the override come
    back. Neither an override nor a revert restarts the processing timer (claim 3), the start tick or
    the progress count. A start-next or a force leaves the suspended slot as it is
    (`quirks.stale_suspended_revert`). An override is **in effect** exactly while a behaviour is
    suspended, however long ago the override was made and whatever has started since; so an
    override that no revert has ended stays in effect across later starts, forces and orders, and
    the revert causes below then apply to whatever object is the actor's target at that time.

    The original overrides when an actor retaliates against an attacker (claim 18; the override is
    to *attack* with the attacker as target and no destination). If an override is in effect, the
    actor reverts on the tick on which any of these happens to its target object:
    - the target is destroyed: the revert comes at the killing hit, on the tick of that hit, also
      when the target stays in play after it (a sinking or crashing object, an infantry unit
      playing its death sequence; glossary, *in play … destroyed, removed*), not later when the
      target is removed,
    - the target goes off the map (for example it enters a transport or a building), or is
      removed. This cause does not include the departure of a superweapon warp member (S-0513
      claim 39),
    - the target makes its teleporting locomotor's own jump (S-0207 claim 20 e), except for an
      attacker whose strike target is the living jumping object (S-0519 Terms,
      S-0207 claim 20 e). Other attackers
      revert if overridden, then drop the jumping object if it is still their target; a revert
      that restores another target keeps that restored target. Projectiles lose the jumping
      target as S-0207 states,
    - the target changes owner (for example by mind control, or a building taken by a building
      taker), except for an actor whose strike target is that living object (S-0519 Terms),
    - the target starts to cloak, or, being of an underground type, goes underground, except
      when the target belongs to the actor's own house or a sensor of the actor's house covers the
      target's cell (Terms, *cloak* and *sensor*).

    Both strike-target exceptions also hold after a strike ends normally: it keeps its strike
    target (S-0519 claims 18 and 20), so no strike need be in progress. They test the strike's
    target, not whether its aircraft still attack that object.

    An actor with an override in effect whose target is a superweapon warp member does not revert
    at the member's departure: it keeps the member as its target and keeps its suspended
    behaviour. While it holds the member, only the causes above revert it. The member's
    destruction by its landing check or by S-0513 claim 38's fate reverts it at the killing hit,
    as the destruction cause gives. A later drop of the member by the actor's own processing does
    not revert it (see below on clearing the target by another rule).

    A mobile actor also reverts when its attack target is exactly the detonation's centre
    cell of a successful area-damage bridge attempt (S-0511 claim 22). The revert restores
    the suspended behaviour and put-aside target and destination. If the restored target
    is still that centre cell, it is cleared; a different restored target is kept. Without
    a suspended behaviour there is no revert, and the matching attack target is simply
    cleared. This event does not create or refresh attack-move memory, nor generally
    erase destinations, queued destinations or remembered cell references. The revert's
    own restoration can nevertheless change the destination. Neighbouring cells and
    objects standing on the centre cell do not acquire this revert cause merely by
    being part of the affected bridge.

    The destruction and off-map/removal causes have no exception apart from that departure: they
    revert the actor whoever owns the target and
    whatever covers its cell. An erasing attacker has no exception to owner-change target loss:
    dropping its ordinary target does not itself detach its beam (S-0514 claim 16).
    The revert brings back the target that the override put aside even
    when that is the very object that caused the revert (an actor that already had its attacker
    as target when it overrode, which claim 18 allows only for owners that are not
    human-controlled): after a revert by destruction, off-map/removal or cloak/underground that object is the
    actor's target again, and after a revert by an owner change or locomotor jump the actor has
    no such target. What the
    actor then does with such a target belongs to the target-acquisition specs: S-0309 claim 27
    gives that the removal of such an object drops it again, and our engine's rule for such a
    target while it stays off the map or in play after its killing hit. Clearing the target by
    another rule, apart from the centre-cell event above (for example the leash of claim 26
    or an order), does not revert. Other users of override and
    revert are outside this spec: team logic, and some movement situations, among them S-0207
    claim 15, where a driving vehicle's repeated step check against an obstacle overrides to
    *attack*, with no destination, against the first object listed in the blocking cell when its
    house is not allied with that object, or against the wall when no object is listed and the
    cell holds a wall (with an allied first object, or with no object and no wall, there is no
    override).
    - Backing: `measurable` (S-0511 scenario B10's target-loss variant for the centre-cell
      revert and restoration; B18 d for the locomotor jump's ordinary target-loss and revert,
      scenario B7: case b for the revert at the killing hit of an attacker
      that leaves play at once, case i for one that stays in play after the killing hit, case j
      for a target that goes off the map and for the target brought back, case k for an owner
      change, and case h for a target
      cleared by an order, which does not revert; cases h and l for the suspended slot kept across
      a start-next and for a later revert caused by another target). These parts are `unmeasured`:
      - underground entry and the own-house exception to cloak target loss: no fixture here
        establishes those states during an override. Enemy cloak start with and without
        the observing house's sensor coverage is instead `measurable` (B45 b),
      - the strike-target exceptions to owner change and a locomotor jump, including after a
        strike ends normally, because each needs a living object held as the strike target at
        the moment of that event, and no setup here establishes that timing. Erasing attackers' owner-change target loss and beam retention
        instead inherit S-0514 claim 16's measurable backing (E10 part h),
      - a warp member's departure, which does not revert, because no part of S-0513 puts an
        override in effect for a holder of a warp member,
      - a removal that follows neither a killing hit nor going off the map, because every removal
        that a setup here can make comes after one of those, which has already caused the revert,
      The suspended slot retained across a force is `measurable` (B45 f): a control-centre
      power transition resets the overridden actor without an ownership change, unlike
      the former takeover-only comparison.
    - Confidence: high (the slot rules, the target replacement, the revert at the killing hit
      and on going off the map, the centre-cell revert and exact-cell matching, and an
      override staying in effect until a revert, so that a later
      target's causes revert it); medium (the owner change, the cloak and underground cause and
      their exceptions, the target brought back, a warp member's departure not reverting, and
      the list of users)

11. **Ready to change, and the per-tick start.** On each tick on which its update runs, an infantry
    unit, vehicle or building that is ready to change and has a behaviour queued makes it current by
    a start-next, its **first per-tick start**. The behaviour that this start makes current is the
    one processed on that tick (claim 3a). For a building this start also needs the building to have
    left its construction state. A building also has a second per-tick start on the same tick,
    without the construction-state condition, which sees the ready mark as that tick's processing
    has left it; a behaviour that the second start makes current is not the one processed on that
    tick, and is first processed on the next tick (claim 3b); see the building rule below. Infantry
    and vehicles also have a **late start**, a further ready-and-start chance in the same update,
    judged with the movement state that update has left. It makes current a queued behaviour of a
    unit that is then ready under the readiness rules below. For a driving vehicle, the test is the
    current predicate below, not a remembered indication that it displaced earlier in the update.
    For an infantry unit, the first start's "in the middle of firing" test (Terms) sees the attack
    state as earlier updates, the orders applied since and other actors' turns left it. The late
    start also sees the endings that this update's processing, its per-tick search (claim 33) and
    its stage-delay-0 test make. Neither start sees this update's firing turn, which comes after
    both. No readiness condition of a vehicle depends on its firing or turret turning. A vehicle's
    body turning is different: see "moving" in the vehicle rules below. A behaviour made current
    there is processed as claim 3b states for a start after the actor's processing (on the unit's
    next update). It is a per-tick start for claim 2's exclusions: it takes no effect on a tunnel,
    erasing or leaving-stage tick, and in our engine none on a tick that S-0513 claim 51 suppresses
    it. The behaviour current at the end of an update is the one that the late start leaves; the
    drop rule and the per-tick search of claim 33 on that tick judged the behaviour as processing
    left it, before the late start, and so never see a behaviour made current late. After the expiry
    update of a berserk unit (S-0610 claim 5), for example, a queued *guard* can become current at
    the late start, but nothing of it (search, hold, processing) happens before the unit's next
    update. Unless something changes the unit's readiness, its slots or its orders between the late
    start and the unit's next update, either start gives the same behaviour for processing on that
    next update. The late start nevertheless has an earlier start tick and changes the current
    behaviour that a recorder or an actor updating later in the same tick can read. A building and
    an aircraft do not have a late start; buildings have the second start above and aircraft the
    end-of-update start below. No quirk switch: it is the original's order, not a defect. Backing
    for the late start. (i) The expiry-update instance of a berserk unit that is infantry, moving,
    and ready: `measurable` by B46 below, which reads the current and queued behaviour after the
    last update of the expiry tick. (ii) An otherwise-ready vehicle whose due *move* processing
    queues ordinary rest: `measurable` by B3 e, which distinguishes the queue at processing exit
    from the current behaviour at update end. (iii) An infantry unit's late start after the
    stage-delay-0 ending of its attack state (the infantry rule below): `measurable` by B45 run c2.
    (iv) Other late-start queued-behaviour cases remain `unmeasured` here: no isolated setup is
    established for their particular combination of readiness changes and queueing. B47 instead
    tests a driving vehicle's first start at the tick-based end of a body turn, not a turn-end late
    start. The empty-slot fall-back to rest below has its own backing. In the unchanged case the
    queued behaviour is also made current by the next update's first start before it is processed,
    so the only difference is its earlier start tick and the state observable between those starts,
    for example by actors updating later in the same tick or by a recorder at update end.
    Confidence: medium (that readiness reads the movement state this update leaves, and for infantry
    the attack state at the points that the infantry rule below gives; the position relative to a
    vehicle's firing and turret turning is not observable and is not asserted). Aircraft have a
    per-tick start after their processing and movement, not before processing: a behaviour made
    current there is first processed on a later tick (claim 3b). S-0501 claim 31 also permits a
    ready aircraft to start its queued behaviour during its locomotor's step, while alive and
    neither landing nor taking off, without a height test, and at the end of an ordinary fall-back
    to rest. The early exits of S-0501 claim 18 do not reach that last opportunity. S-0501 claim 31
    owns all three aircraft start points. An infantry unit that is ready to change and has neither a
    current nor a queued behaviour falls back to rest (claims 13 and 14) at its first per-tick
    start, and the behaviour that this fall-back queues, if any, becomes current at that same start.
    The late start does the same: an infantry unit that is ready at the late start with neither a
    current nor a queued behaviour falls back to rest in that same update, and the behaviour that
    the fall-back queues, if any, becomes current at that same late start, so a recorder reading
    after the update sees it current (not queued, and not deferred to the next update). That
    update's firing turn sees the state the fall-back leaves. Like any late start it is not
    processed until the unit's next update. Backing: `unmeasured`; no setup is known that leaves an
    infantry unit ready with both slots empty at the late start of a chosen update (claim 22's
    fall-back happens inside a processing, which leaves a behaviour current or queued, and a newly
    created actor is the only case that starts with none), so the claim cannot be observed without
    inventing one; confidence medium. An infantry unit that completes a capture or hijack is
    consumed in that update, so no observable depends on whether the late start follows.

    These per-tick starts, the late start included, are subject to claim 2's exclusions. In our engine, S-0513 claim 51
    additionally suppresses them for vehicles, teleporter infantry and landed aircraft from
    the first pending warp update through arrival effects, before and after warp work. During
    arrival hold they regain ordinary eligibility after the hold-end test, whether the hold
    continues or expires. Readiness alone cannot start a queued behaviour on an excluded turn.
    This is an engine design rule, not a resolved original continuation rule (S-0513 claim 52).
    Readiness depends on the actor kind:
    - **Infantry** is ready unless any of these holds: its current behaviour is *sticky* or
      *rescue*; it is in the middle of firing; it is falling (for example from a parachute); it is
      moving on this tick while its current behaviour is not *guard* or *hunt* and is not *attack*
      without a target; or its current sequence (S-0107) is a kind that is not interruptible.
      **In the middle of firing** (Terms) is S-0107 claim 11's attack state and nothing else. The
      rearm countdown (S-0301), a firing kind that is still playing and a shot in flight do not
      hold readiness back by themselves.
      - *Begins* in the unit's firing turn on a tick on which it has a target, is not in the
        attack state and its firing check answers *can fire*, with S-0107 claim 11's request
        for a firing kind. It begins whether that request is taken or turned down (for example
        because the kind requested is the one already current), and with the airstrike reset in
        place of a request. It does not wait for the shot.
      - *Ends at the shot.* When the weapon fires at the fire frame *F*, the attack state ends
        with that shot, in the same firing turn. With *F* = 0, when the request is taken, it
        begins and ends in one firing turn, so no per-tick start sees it.
      - *Other endings* are S-0107 claim 11's, each at its own point:
        - a fire frame at which the firing check does not answer *can fire*, and no target at
          the firing turn: in the firing turn, so both starts of that update still see the
          state.
        - a current sequence whose stage delay is 0: after the update's processing and before
          its late start, so only the first start of that update sees the state.
        - the airstrike repeat of a completed secondary firing kind (S-0107 State and ordering), for
          a unit still in the attack state when that kind completes (S-0107 claim 11 gives when it
          is): after the firing turn, so both starts of that update see the state.
        - a change of its target, to another target or to none, by an order (a move, deploy or
          stop order, or an attack order on another target) or because the target leaves play:
          when the order takes effect or the target leaves, so no start after it sees the state.
        - a change of its target made by its own processing, for example the drop of S-0517 claim
          16, or by claim 33's per-tick search, which runs after the processing: in that processing
          or search, so the first start of that update sees the state and its late start does not.

        A change of target also asks (not forced) for *deployed* when deploying, for *prone* when
        prone and for *stand* otherwise, as the not-ready fire frame does, whether or not the unit
        is in the attack state at that change (S-0107 claim 6 decides whether the request is taken),
        so a firing kind still playing gives way to it. The attack-move rule of S-0309 claim 23
        discards a target, and puts a remembered target back, without this change-of-target ending.
        Where it discards the target and its search finds no other, the attack state lasts to the
        no-target ending in the firing turn. Where it puts the remembered target back, the state
        stays as it was. A target that its search finds is a change of target as above.
      - *A unit that keeps its target* is in the attack state from the firing turn that requests a
        firing kind to the shot. After the shot it is not in the attack state until its firing check
        answers *can fire* again, which is at or after the end of the rearm countdown and does not
        depend on the end of the firing kind. If that happens while the firing kind is still
        current, the attack state begins again with no new sequence (the request is turned down) and
        lasts to one of the endings above. Unless the airstrike repeat ends it when the kind
        completes, that is:
        - for a standing unit, the stage-delay-0 ending in the first update that begins with
          *stand* current after the kind completes, so a behaviour queued meanwhile becomes
          current at that update's late start and is first processed at the next update.
        - for a prone unit, the attack state lasts into *prone*, to S-0107 claim 11's repeat
          shot or its not-ready fire frame.

        If the rearm countdown outlasts the firing kind, the attack state has ended at the shot,
        and the next shot comes from a new firing kind.
      - No quirk switch of this spec: it is the original's order, not a defect. The prone bullet
        above holds with S-0107's `quirks.prone_attack_state_persists` on, its default. With it off,
        a prone unit's attack state ends when *fire-prone*, or *secondary-fire-prone* that is not
        repeated for an airstrike (State and ordering), completes. The rest of this block holds in
        both settings.
    - **Vehicles** are ready unless any of these holds:
      - the current behaviour is *sticky* or *rescue*,
      - the vehicle is in one of exactly three transitions: it is deploying, it is undeploying
        (the deploy specs own when each begins and ends), or it is in the middle of letting its
        passengers out under its own *unload* processing (the transport specs own that
        processing). A vehicle that has finished deploying and stays deployed is not held back
        by this condition,
      - the vehicle is moving on this tick and is not below the surface (its height above the
        surface is not negative, so a vehicle travelling underground does not meet this
        condition). This asks the vehicle's current locomotor for its *moving now* report.
        While a temporary replacement is installed, that replacement's report applies.
        In particular, a seized unit uses temporary jumpjet movement, not its saved
        driving movement (S-0607 claim 20). For a vehicle whose current locomotor is
        the driving one, "moving" here means either
        that its body facing is still turning for any reason, or that all three of these hold:
        its locomotor is moving it in S-0206's sense (Terms), its heading point is present
        (S-0207 Terms), and its integer movement step *s* of S-0207 claim 2 is strictly
        positive. That step is the integer progress earned per tick, not the speed fraction
        and not a count of coordinates actually traversed. Its computation and truncations
        remain S-0207 claim 2's. This readiness test is distinct from S-0206's term:
        a retained locomotor destination alone is insufficient, while a body turning without
        any movement destination still counts.

        A body turn ends by tick number, as S-0206 claim 1 states. If its unchanged turn
        starts at tick *t*₀ and lasts Θ ticks, it is no longer turning at either start of
        tick *t*₀ + Θ. With no remaining positive-step movement condition or other blocker,
        the queued behaviour therefore becomes current at that tick's first start and is
        processed that tick (claim 3a), not first on the next tick. The late start does not
        add a tick to a completed turn. A new turn or movement condition arising during
        the update can still affect the late start independently.

        Thus an otherwise ready vehicle standing in *area guard*, *sleep* or *attack* with
        a target, with no heading point and no body turn, starts an ordered *move* at its
        next first start even though destination assignment has already supplied a locomotor
        destination (S-0207 claim 25). A blocked or retry-waiting vehicle is likewise ready
        when no body turn remains and its heading point is absent or *s* is zero. Neither
        a retained goal nor unused path steps alone holds it back. Conversely, a retained
        heading point together with positive *s* and the locomotor-moving condition still
        blocks readiness even without visible displacement. These are current-state tests,
        not a blanket exemption for every blockage or retry wait. The exceptions below apply to a turning vehicle in
        the same way as to a travelling one. The condition does not hold in three cases: the queued behaviour is *enter*; the current behaviour
        is *guard*, or *attack* without a target; or the queued behaviour is a *guard* that one of
        two *unload* rules queued with an allowance to interrupt movement. Those two rules belong to other specs: a transport
        vehicle that reaches the last stage of unloading its passengers queues *guard* (S-0507
        claim 11), and a jumpjet vehicle in *unload* that is sent off again while it descends to land
        queues *guard* (movement specs). The allowance lasts until the slots next change (any
        queue that takes effect, start, force, override or revert ends it). Confidence: high for the driving readiness predicate and tick-based turn-end start.
        Backing: `measurable` by B47 for the isolated body-turn condition and same-tick
        processing, and B48's fresh-move cases for a destination without a heading point.
        Readiness under magnetic seizure is `unmeasured` here: no fixture establishes
        a distinguishable queued behaviour during a qualifying airborne state.
        S-0607 claim 20 supplies the current-locomotor rule, with confidence high.
        The retained-goal blockage/retry comparisons, including a present heading point with
        zero or positive integer movement step and no displacement, are `unmeasured`: B48
        establishes no legal order that both preserves the required waiting state and leaves
        a distinguishable behaviour queued. Its retained-goal part is a conditional investigation,
        not backing for those branches. The other rules of this condition
        keep the backing given in the Backing bullet below,
      - its unload or deploy cycle has not finished (never holds for vehicles, S-0517 claim 15),
      - it is in contact with a vehicle-factory building and the queued behaviour is neither *move*
        nor *enter*,
      - it is in contact with nothing and stands on a cell of a vehicle-factory building that is the
        cell (*x*, *y* + 1), where (*x*, *y*) is the top-left cell of that building's foundation
        (the cell where a new vehicle leaves the factory).
    - **Aircraft** readiness is S-0501 claim 31's rule: not in *sticky* or *rescue*, no run lock
      except in *spy-plane approach*, and no processing-cleared aircraft ready mark. The run lock
      is the glossary's term; the aircraft ready mark is distinct from the building mark below.
    - **Buildings** are ready while their ready mark (Terms) is set. Every processing of an armed
      building in *guard*, *sticky* or *area guard*, and every processing of *attack* by a building
      without the SAM flag that has a target, sets the mark (claim 32). The construction state
      also sets it (S-0406 claim 17). For a building without a turret, the body range sets it
      whenever the stage advances onto its last stage, or on every non-advancing turn of a
      zero-delay range. For a turreted building this body-range rule applies only while its
      behaviour is *construction* or *selling*, including a queued one seen while none is current
      (claim 1). Running past the last stage and wrapping does not itself set the mark.
      Factory release and unloading neither set nor clear the mark. The factory's queued
      *unload* and the *guard* queued at its end therefore wait for the same ready-mark rule
      (S-0417 claim 2, `quirks.factory_unload_waits_ready_mark`, default original; that spec
      owns the alternative). For an unarmed factory without a turret, its body range supplies
      the mark. A building's grand opening (Terms) also sets it; at game set-up this is what
      makes the queued *guard* current at the building's first per-tick start (claim 1). Other structures rules
      (repair, selling and others) also set or clear it.
      Each of the building's two
      per-tick starts clears the mark when it succeeds; nothing else of this spec clears it. So a
      behaviour queued for a building becomes current at the first of its per-tick starts at which
      the mark is set (and, for the first start of a tick, the building has left its construction state); this
      can be the second start of the very tick whose processing set the
      mark, and a behaviour made current by that second start is first processed on the next tick
      (claim 3b). The starts that a building's own processing makes (claim 32) are made whether or
      not the mark is set and leave it as it is. Consequence: an armed building that has processed
      *guard* at least once since its last successful per-tick start takes a queued behaviour (for
      example an ordered *attack*) at its next per-tick start, without waiting for its next
      processing.
    - Backing: the warp start exclusion and hold resumption are `measurable` in our engine (S-0513
      scenario SW23 part f, extended by part g); original continuation beyond the leaving-stage
      exclusion remains `unmeasured` under S-0513 claim 52's incomplete per-kind evidence. Other
      rules are `measurable` (scenario B3, cases b, c, g and h; scenario B23 for the vehicle
      exceptions of queued *enter* and contact with a vehicle-factory building, and case c for a
      transport letting its passengers out; scenario B16, case d, for buildings; S-0406 PL11 for the
      construction ready mark and second start; scenario B35 for the body-range rule, and S-0417 U2
      for factory release and unloading leaving the mark alone). Further parts are `unmeasured`.
      Starting *move* inside a deploying or undeploying transition has S-0517 claim 10's
      unreachable-input qualification. The unload or deploy cycle is always finished for vehicles
      (S-0517 claim 15, measurable in P6), so it never blocks readiness. The vehicle below the
      surface: no setup here puts one there. The two *unload* rules with the allowance to interrupt
      movement: they fire only at the last stage of an unloading or of a descent, which belong to
      the transport and movement specs, and no setup is known that has the vehicle moving at that
      instant. Aircraft readiness and queued starts have S-0501 claim 31's backing: AC10 tests
      ordinary flight, take-off, landing and run-lock timing; its remaining exclusions are
      `unmeasured` for the reasons that claim gives. The building's second per-tick start is
      measurable at construction entry under S-0406 claim 17 and PL11. The infantry falling
      condition is `measurable` with fresh paratroopers (S-0522 claim 17 and scenario B44):
      placement already queues rest, so no accepted order during descent is needed to test
      readiness. A move click during the fall gives a move order at once, which starts after landing
      (S-0522 claim 17, at its confidence). Being in the middle of firing is `measurable` by B45 c:
      runs c1 and c2 for its begin, its end at the shot (not at the end of the firing kind or of the
      rearm countdown), and its second begin while the kind still plays with the late start that
      follows; run c3 for the end at an order's change of target. The request that a change of
      target makes is S-0107 claim 11's, with its backing for an order's change of target on a
      standing unit that does not move, in whichever of the two states the order finds it (S-0107
      scenario M8 part d). That it is made both in and out of the attack state, for a moving unit,
      and at a change of target that is not an order's, is `unmeasured`: that part does not record
      the attack state at the order, gives no moving unit and gives only orders. For a deploying or
      prone unit it is `unmeasured`: no scenario changes the target of a firing unit that is
      deploying or prone. A sequence that is not interruptible is `measurable` by run c4. In each
      run, scripted input places an order at a recorded point of the firing cycle, and a read-only
      recorder records when the queued behaviour becomes current and whether the update that makes
      it current processes it. The other endings of being in the middle of firing (the not-ready
      fire frame, no target at the firing turn, a sequence with stage delay 0 other than after a
      completed kind, the airstrike repeat, a change of target by the unit's own processing, a
      target leaving play, and the attack-move rule) are `unmeasured` here: B45 c isolates none of
      them from the order route. They keep S-0107 claim 11's backing where it gives one. The begin
      with the airstrike reset in place of a request is `unmeasured` here as well: no run of B45 c
      designates an airstrike.
    - Confidence: high (the engine warp-start design contract, the infantry conditions other than
      being in the middle of firing, and the vehicle conditions on *sticky* or *rescue* and on
      moving); medium (being in the middle of firing: that it is the attack state, its begin, its
      end at the shot, its end at an order's change of target until run c3 runs, its other endings
      and their places in the update, its second begin while the kind still plays and what follows
      it, the request that a change of target makes, and the attack-move rule); low (matching the
      original's unresolved warp-start continuation); medium (the list of three vehicle transitions,
      the unload or deploy cycle, the vehicle-factory conditions and buildings); low (the two
      *unload* rules, the vehicle below the surface); aircraft readiness and start timing have
      S-0501 claim 31's confidence

11a. **Failed departure while harvesting.** A non-teleporting harvester leaving a
    refinery's docking cell has no special behaviour-start or retry rule. The failure
    categories of S-0212 claim 23 decide whether *harvest* survives:

    | Failure category | Behaviour consequence |
    |---|---|
    | The path-computation gate refuses, or a failed search does not give up | No behaviour is queued by the failure. If *harvest* remains current, seeking with neither destination nor remembered cell searches again at its next processing under S-0402 claim 3. |
    | The failed search gives up | The failure queues *guard* for a human-controlled owner, or *area guard* for a non-human-controlled owner. In our engine every player house uses the human-controlled rule; neutral and special houses retain the non-human-controlled rule. |

    Queueing here requests no immediate start. The current behaviour and processing
    timer are not changed by that queue request. The queued behaviour starts at the
    next per-tick start at which claim 11 permits it, without waiting for the old
    processing timer or path retry delay to expire. Its start expires the processing
    timer. A first per-tick start permits processing in that turn (claim 3a); a late
    start instead leaves first processing to the next update (claim 3b).
    A failure after that turn's first per-tick start cannot use that already-passed start,
    but can still reach its late start if ready there.
    Until a start occurs, any processing still uses the current behaviour. Thus
    *guard* need not already be current on the failure tick, but a pending *guard*
    must not be treated as a promise of another *harvest* processing. Once *guard*
    starts, the seeking rules no longer run merely because the old destination was
    cleared. Any later resumption of harvesting follows S-0402 claim 13 or a new order.

    The non-human-owner give-up destination is assigned on the failure tick under
    S-0212 claim 23a and claim 38 here, not deferred until *area guard* processes.
    It does not queue *move* or force *area guard* to start.
    - Backing: `measurable` (B38; S-0402 H20 and S-0212 claim 23's failure scenarios)
      for queueing and first-start timing. A failed-departure late start remains
      `unmeasured`: B38 does not establish a failure that leaves the harvester ready
      at that late start. Its conditional late-start recording is not such a fixture.
    - Confidence: high for queue/start separation and the absence of a docking-cell
      exception; medium for the path-failure categories, following S-0212 claim 23

12. **Orders go through the queue.** A player's order that names a behaviour (move, attack, guard,
    area guard, enter, capture and the other orders that carry a behaviour), carried by the order
    system, queues the ordered behaviour without an immediate start; the order also sets the
    target, destination and guard spot it carries. The behaviour therefore becomes current at the
    actor's next per-tick start (claim 11; for aircraft, through the aircraft rules that claim 11
    points to). For infantry, vehicles and buildings, this is the actor's first update after the
    order takes effect if the actor is ready at that start, otherwise the first later permitted
    start. Aircraft use the three opportunities of S-0501 claim 31, including their
    end-of-update start after processing; starting there does not process the new behaviour
    in that same turn. In a single-player
    game (a skirmish or a campaign mission) an order takes effect after the turn pass of one tick
    and before any object's turn in the next (S-0203, claim 7), so this is the actor's update on
    the next tick at the earliest; network games are out of scope here, as in S-0203. An order for the behaviour
    that is already current changes no slot (claim 7); its new target or destination takes effect
    directly.

    An attack-move order is first turned into a plain behaviour order by S-0213 (human orders,
    claim 6), and the behaviour that results is queued as above. In short: a mobile actor whose
    type is eligible for attack-move gets *move* (with the attack-move memory) if the order carries
    a destination, else *attack* if it carries a target, else *attack move* itself (claim 5); an
    ineligible mobile actor gets *attack* if the order carries a target, else *move*; a building
    gets *attack move* itself. S-0213 owns this translation and its backing; this spec adds only
    that the result goes through the queue like any other ordered behaviour. In particular,
    ordinary eligible target-only and destination-only input queues *attack* and *move*,
    respectively, not the standing behaviour *attack move*. No ordinary-input path studied
    establishes that standing behaviour (claim 5). Losing eligibility before application can
    expose the ineligible-mobile translation without creating a building order or an order
    with neither reference (S-0213 claim 6).

    Every such order also forgets the target and destination that an override put aside (claim 10),
    but leaves the suspended slot as it is (`quirks.stale_suspended_revert`). For a mobile actor it clears the guard spot and the queued
    destinations, except an area-guard order, which takes its object or cell as both the guard spot
    and the destination and clears the target.

    The **stop order** is not a behaviour order: it changes no slot and forces nothing. It clears
    the actor's target, destination, queued destinations and route, ends its contacts, and leaves
    its guard spot alone. Two exceptions: a harvester vehicle whose current behaviour is *harvest*
    or *return* queues *guard* and starts it at once, whether or not it is ready to change, and a
    slave master also gives its harvesting slaves a guard reset. The stop order does nothing to an
    actor whose current behaviour is *construction* or *selling*. As a consequence, an actor in
    *move* or *attack* that has been stopped falls back to rest at a later processing (claims 22 and
    23), once it has stopped moving. An infantry unit in *move* instead falls back at the end of the
    update after which it has stopped moving (claim 22, end-of-update check), unless a *move*
    processing already finds it stopped, with no destination and nothing queued, and falls back
    first by claim 22's general rule. The order specs own the stop order's further conditions.

    **Destination clears for jumpjets.** S-0502 claim 24 governs every effective destination
    clear in this spec, including the fall-back outcomes of claims 13 to 15 and the
    processing outcomes of claim 22, regardless of the caller. A committed jumpjet
    reroutes and retains its replacement movement point while its order-level destination
    becomes empty; this is not a direct instruction to move to an absent point. A
    behaviour change alone is not a destination clear. Infantry of a jumpjet type has no
    unchanged-destination suppression: even an empty-to-empty assignment reroutes while
    a movement commitment remains, including when behaviour processing makes that
    assignment. Each effective assignment can repeat the eligible standing-position draw
    (S-0502 claim 24). An unchanged vehicle destination assignment suppressed under
    S-0502 claims 24 and 27 does not reroute or erase the retained movement point.

    A balloon-hover vehicle can instead reject a requested clear under S-0502 claim 28,
    including one requested by behaviour processing. The rejection retains the nonempty
    order-level destination, movement point, commitment and landing reservation, with no
    reroute or movement-position draw; it does not substitute another destination. Unless
    its current behaviour, or queued behaviour when no current behaviour exists, is already
    *attack*, the rejection forces *attack*, clearing the queued behaviour and applying
    claim 9's behaviour-start effects. A force within processing still has claim 3c's delay
    outcome. These assignment rules qualify every destination-clearing outcome below:
    the requested clear is not a guarantee that the destination becomes empty or that
    the preceding behaviour remains current. S-0502 claims 24, 27 and 28 own the
    no-commitment outcome, refresh conditions and rejection predicates.

    **Spawners' own targets.** Target changes made by these orders, and by the fall-back and
    behaviour rules in this spec, obey S-0506 claim 25. With its quirk at the original setting,
    actually changing a nonempty own target to empty requests a pending-target clear, effective
    only if the controller target is nonempty. An already empty own target leaves pending
    unchanged, as does direct replacement by another nonempty own target. Restoring an own
    target does not restore pending that was cleared. A behaviour name or a fall-back to rest
    is not itself a cancellation, and a targetless actor is not swept for pending cancellation
    on every tick or launch pass. S-0506 claim 25 owns the alternative quirk setting,
    target-loss notifications and explicit controller resets.
    **Strike designators' targets.** Every actual designator target change caused by an
    order, fall-back or behaviour rule here cancels its strike immediately under S-0519
    claim 19, including a change to no target. Setting the same target again does not cancel.
    Changing away and back within one tick still cancels; restoring the target does not undo
    cancellation. This happens when the target change applies, not at a later strike update,
    behaviour processing or designation impact.
    - Backing: `measurable` (scenario B3; S-0519 scenario T18 for immediate strike cancellation,
      scenario B14, case b, for the two attack-move orders that
      input creates; S-0506 scenario R19 for actual own-target changes and pending cancellation,
      and S-0502 claim 24's scenarios J19, J22 and J23 for effective jumpjet destination
      clears and repeated infantry assignments, claim 27's J24 for vehicle refresh, and
      claim 28's J25 for balloon-hover retention and its possible *attack* start, inheriting
      those claims' unmeasured limits for combined-type and refresh fixtures). Attack-move orders with no established input path (neither target nor
      destination, or a building) are `unmeasured` here: the measurement setup cannot issue them.
      Translation after loss of eligibility is also `unmeasured` here; S-0213 claim 6 owns that
      case and the translation's backing. The stop order's exemption for
      *construction* and *selling* is `unmeasured`: a building has no destination, queued
      destinations or route for the stop order to clear, and whether the input can give the stop
      order to a building that is being built or sold is not established; a mobile actor carries
      these behaviours only when data places it in them (claim 4), and any order that gives it a
      target or a destination also queues a behaviour that replaces them at its next per-tick
      start, so no state is known in which the exemption would show.
    - Confidence: high (queue without start; the stop order changes no slot; pending-clear conditions); medium (the effect on
      a moving vehicle, which follows from claim 11, the clearing rules and the stop order's
      exceptions)

13. **Falling back to rest: common effects.** An aircraft first checks the three early-exit rows
    of S-0501 claim 18: restoration of an overridden behaviour; keeping *retreat*, either
    paradrop behaviour or either spy-plane behaviour when it is not an airstrike aircraft,
    or starting queued *paradrop approach* or *spy-plane approach*. A matching row ends that
    fall-back without any common effects below, including refresh, and without the ordinary
    end-of-fall-back queued start. S-0501 claim 18 owns those rows and their backing.
    Any actual own-target changes during fall-back use claim 12's spawner rule, not
    unconditional pending-target cancellation.

    Otherwise, when an infantry unit, vehicle or aircraft falls back to rest, the effects of
    this table take place, and the tables of claims 14 and 15 (for aircraft, S-0501 claim 18)
    decide on the state they leave. In particular, a
    destination that one of these effects gives counts as the actor's destination there, so that the
    table's destination row applies.

    | Situation | Effect |
    |---|---|
    | its own locomotor has been temporarily replaced by another and that replacement can end | it gets its own locomotor back; S-0607 claim 20 owns seized-jumpjet restoration eligibility, its end-of-turn opportunities and its exclusions; S-0417 claim 5 establishes temporary driving during factory exit; S-0207 claim 21 owns the restored teleporting state |
    | it is a vehicle with approach destinations remaining after refresh | the first of them becomes its destination and is removed from the list; the route row and the guard-spot row do not apply in this fall-back |
    | it follows a route, the approach row does not apply, and it is not in a *patrol* that has already begun (progress count not 0) | it queues *wait* with an immediate start and issues the action of the next route point as an order (S-0213 claim 16c); this is not an immediate destination assignment |
    | it is an infantry unit that is not a harvesting slave, its guard spot is on another cell, the approach row does not apply, and it has not got its own locomotor back in this fall-back | the guard spot becomes its destination; unless its current behaviour is *area guard*, it also queues *move* and forgets the guard spot |

    Restoration retains the suspended movement state, rather than creating an empty one.
    For teleporting vehicles S-0207 claim 21 distinguishes a retained pending jump, landing
    point and phase-in expiry from the destination changes that the remaining fall-back
    effects may then make. Retention at restoration does not exempt a move from replacement
    or cancellation by those later effects.

    The approach row prevents the route and guard-spot rows. Otherwise the guard-spot row's
    destination survives this fall-back. A route point issues an order through the ordinary
    order timing of S-0213 claim 1, so its destination and behaviour do not replace the actor's
    state within this fall-back. The issue is preceded by S-0213 claim 16c's queue of *wait*
    with an immediate start, so for a holder that is ready to change (claim 11) *wait* is
    current when the rest tables of claims 14 and 15 run, their row 1 applies, and the fall-back
    queues no *guard*. Its later application is governed by the order specs. This corrects the
    earlier claim that the next route point immediately supplied a destination.

    **Repair/reload route points.** For a route point resolving to an object, the action is
    classified as an ordinary object click (S-0214 claims 4, 7 and 8), with the actor owner's
    house as the acting house. Under the owner-routes departure, all alliance tests in this
    paragraph use that same owner, not a house played on the machine. When the actor owner's
    house counts the object's owner as allied, an object-directed action is used only for a
    building that meets at least one of these service conditions:
    - it is a unit repair building, the actor's integer health is below its type's maximum health,
      and the actor is not infantry,
    - it is a unit reload building, the actor's integer ammunition count is below its type's
      maximum ammunition and that count is not −1 (unlimited ammunition).

    In addition, the building's owner must count the actor owner's house as allied,
    and the classified action must be neither *select* nor *toggle select*. Thus this service
    branch requires mutual alliance with the actor owner's house, not merely a one-way alliance. When these guards hold, the point issues
    the classified object action as an order, rather than replacing it with a nearby-cell move.
    Failure of any guard uses the cell-action fallback instead. For an object that the actor
    owner's house does not count as allied, *select*, *toggle select*, *move* and *no-move*
    actions likewise use the cell fallback; other classified actions remain object-directed.
    A point with no object also uses the cell action. These are route-order selection rules,
    not permission to pass through an occupied repair building or a guarantee of docking or
    paid service (claim 29). The issued order still takes effect at the ordinary order boundary.
    The original instead uses the house played on the machine for both alliance guards and
    the non-allied fallback test. Our engine has no machine-local simulation house, including
    in a replay or a skirmish without a person playing: no substitute local house is needed.

    The common effects never send a
    scatter request. The scatter that can follow a fall-back to rest in *capture*, *sabotage* and
    *eaten* is sent by that processing (claim 28), not by the fall-back.

    These effects take place at most once per actor per tick: in a second fall-back to rest on the
    same tick none of them happens, and only the rest-choice rules of claim 14 or 15
    (aircraft: S-0501 claim 18) apply. An actor with an attached temporal beam also releases
    that beam as a common fall-back effect (S-0514 claim 14). Release shares this once-per-tick
    limit and is skipped by the aircraft early exits above. It tests for an attachment, not
    merely a temporal weapon. Stop-order target clearing alone, or assigning or processing
    *stop* or *sleep*, does not cause this release. A later fall-back can do so; no release
    in the stop order's application tick is promised. Attack-move memory is refreshed before the table's effects:
    S-0309 claim 23 (b) owns removal and resumption. The rows judge the current behaviour and
    destinations left by that refresh, including any readiness-dependent immediate start.
    Row 1 of claims 14 and 15 sees only memory that survived the refresh. The refresh shares
    the once-per-tick limit and is not part of the per-tick start of claim 11.

    **Collisions with attack-move refresh (Q-0202).**
    - An eligible infantry guard-spot row replaces a resumed attack-move destination with the
      guard spot. Unless *area guard* is still current, it queues *move* without an immediate
      start and forgets the guard spot. A resumed *move* that already started remains current,
      now directed to the guard spot. If readiness prevented the resumed start, *move* remains
      queued and the previous behaviour remains current. With target-only memory, the resumed
      *attack* target is kept, but the guard-spot row can queue *move* instead: an already-started
      *attack* remains current with *move* queued, or the queued *attack* is replaced by *move*
      if it did not start. Claim 7 still governs refusal and redundant queue requests.
    - For a vehicle, the approach row uses the list left after refresh, not a saved copy from
      before it. Resuming a different destination assigns that destination by the vehicle's
      normal rules, discarding the previous approach entries; any entries supplied by that
      assignment are the list the row sees. Resuming target-only memory assigns no destination,
      so a surviving approach entry can become the destination. That row itself queues no
      behaviour: the resumed *attack* is current if it could start, queued otherwise.
    - A route row that remains eligible issues its next-point order even when memory survives.
      Within this fall-back it leaves the refreshed destination and target alone, and changes
      the slots only by the queue of *wait* that precedes the issue (S-0213 claim 16c); an
      eligible infantry guard-spot row can still replace the destination. A later route order
      takes effect at the order boundary, not as a second refresh or an immediate start of the
      order's behaviour here (the *wait* before the issue has S-0213 claim 16c's immediate
      start).
    - Removing memory because the actor is neither engaged nor in *guard* does not suppress
      the approach, route or guard-spot rows. They apply as without memory; the ordinary rest
      table then judges their resulting state. Removal alone queues no behaviour.

    **Whose route (original only).** In the route row, the original takes the route's next point
    from the routes of the house played on the machine running the game, not from those of the
    actor's owner. **In our engine** the next point comes from the owner's routes on every machine
    (a departure, Edge cases). The two differ only for an actor that follows a route while its
    owner is not the house played on the machine. Routes are laid out by player input (S-0213,
    claim 16), so in a skirmish only the human-controlled house, the one played on the machine,
    lays any out; an actor of another house could follow one only after an owner change in the
    middle of a route (for example a mind-controlled vehicle), and whether an owner change keeps
    the route is not established (Open questions).

    **Skirmish distinction (Q-0663 question 3).** For an actor owned by the house played on the
    machine, the two alliance readings are identical, even if the point object's owner changes
    or alliances are asymmetric. They can differ only when actor owner and that house differ,
    and then only when the relevant directed alliance tests differ and affect the chosen action.
    No additional reachable case is established for the original's ordinary single-person
    skirmish beyond the conditional owner-change case above. In our engine an AI house can
    issue the same legal route orders as a person (S-0213 claim 16; S-0214's acting-house rule),
    so an AI-owned route needs no ownership change to distinguish the readings. For example,
    a damaged AI-owned vehicle routing to its own repair building, with a non-allied person
    playing another house, qualifies for the owner-based service branch; using that person's
    house instead fails the reverse-alliance guard even if the ordinary classification is
    *enter*. This is a design consequence, not an additional observed route case in the original.
    - Backing: `measurable` (S-0514 scenario E10 for attached-beam release and the distinction from
      target clearing and stop/sleep processing; scenario B15: cases a and d for the guard-spot
      row, case b for the route row's eventual movement; B15 e and S-0417 U6 for temporary driving
      restoration, B33 for route-order timing; B40 for repair/reload route-point selection; S-0309
      scenario A6, parts g, n and o, for attack-move refresh preceding the rest table and its
      resumption or removal outcomes). The original's choice of the local player's routes over the
      owner's is `unmeasured`: it differs from the owner's routes only for an actor that follows a
      route while its owner is not the house played on the machine. Network games, where that is
      common, are out of scope here (S-0202, claim 12); in a skirmish the only known way is an
      owner change during a route, and since it is not established that a route survives an owner
      change, no setup is known that is sure to produce the case. The approach row and its refresh
      collision are `unmeasured`: no known input reliably provides specified approach entries at a
      fall-back. Temporary driving restoration is measurable for a factory-produced teleporting
      vehicle. Magnetic-lift restoration, its end-of-turn opportunities and its exclusions keep
      S-0607 claim 20's backing and confidence, branch by branch (its being-erased case is
      `unmeasured`, low); other temporary replacements remain `unmeasured` without an established
      setup. The guard-spot/memory and route/memory collisions are `unmeasured`: ordinary
      attack-move input clears the guard spot and ends the route, and no stock input sequence
      reliably keeps both states. B33 gives conditional recorder checks, not a way to manufacture
      them. Also `unmeasured`: the route row's condition on a *patrol* that has begun, because the
      *patrol* processing belongs to other topics (claim 34). Route-action guards involving
      asymmetric alliances or owner-changed route holders are also `unmeasured`: no established
      skirmish input preserves such a route and isolates those classifications. B40 does not
      establish those guards. Also unmeasured is the rule of at most once per tick, which needs two
      fall-backs of one actor in one tick, which no setup is known to time. Scenario B15, case d,
      backs the guard-spot row for a unit whose current behaviour is not *area guard*. The queue of
      *wait* before a route point's issue, and the *guard* that the fall-back then does not queue,
      have S-0213 claim 16c's backing (`measurable` by S-0213 Scenario O8, part t at P₁).
    - Confidence: medium (the effects and which destination remains, and that the original uses the
      local player's routes); high (that no scatter request is sent, and that memory refresh
      precedes the destination rows and rest table), medium for route-order timing, for the queue of
      *wait* before a route point's issue and for vehicle assignment's list effects; the memory
      lifecycle has S-0309 claim 23's confidence; medium (factory-exit restoration and its
      guard-spot exclusion); magnetic-lift restoration has S-0607 claim 20's confidence; low (other
      temporary replacements and which rules fill the approach destinations)

14. **Falling back to rest: infantry.** When an infantry unit falls back to rest, after the common
    effects of claim 13 the result is the first matching row:

    | # | Situation | Result |
    |---|---|---|
    | 1 | the unit has attack-move memory, or its current behaviour is *wait* | nothing changes |
    | 2 | it has a target | queue *attack*, except that a current *sabotage* or *capture* is queued again instead |
    | 3 | it has no destination and its queued destinations are not empty | the first queued destination becomes its destination (the list may keep it at the end, for a looping route), and the unit queues *move* as in row 4, except that a current *sabotage* or *capture* is queued again instead |
    | 4 | it has a destination | queue *move*, except that a current *sabotage* or *capture* is queued again instead |
    | 5 | its current behaviour is *guard* or *area guard* | nothing changes |
    | 6 | its current behaviour has the zombie or the paralysed property | nothing changes |
    | 7 | **original only:** computer-controlled owner, not a team member, and the owner's house skill level is below the area-guard threshold | queue *guard* |
    | 8 | **original only:** computer-controlled owner, not a team member, and the unit is a harvesting slave | queue *guard* |
    | 9 | **original only:** computer-controlled owner, not a team member, and the unit is armed, an engineer or a vehicle thief | queue *area guard* |
    | 10 | **original only:** computer-controlled owner, not a team member | queue *guard* |
    | 11 | human-controlled owner, not a team member, and the unit has the guard-area veterancy ability or its type defaults to area guard | queue *area guard* |
    | 12 | otherwise (human-controlled owner, or a team member) | queue *guard* |

    Whatever the row, nothing is queued while the current behaviour is *patrol* or *area guard*: an
    infantry unit in *area guard* that falls back to rest with a target does not queue *attack*.
    Every "queue" of this table is a queue without an immediate start (Terms, queueing wording).

    **In our engine** rows 7 to 10 never match and no unit is a team member (Terms, houses in our
    engine), so the infantry of every player house, AI houses included, rests by rows 11 and 12: in
    *area guard* if it has the guard-area veterancy ability or its type defaults to area guard,
    otherwise in *guard*. The neutral and special houses keep rows 7 to 10 (Terms, houses in our
    engine) at house skill level 0 (S-0412, claim 3), so their infantry not in a team rests in
    *guard* by row 7 whenever the area-guard threshold is above 0, in the original and in our
    engine.

    **Original only (note for AI designers).** In the original, an infantry unit of a
    computer-controlled house that is not in a team rests in *area guard* when it is armed, an
    engineer or a vehicle thief, and in *guard* otherwise; it rests in *guard* in every case when it
    is a harvesting slave or when the house's skill level is below the area-guard threshold (rows
    7 to 10). Our AI can get the same result by ordering its resting infantry to guard an area.
    - Backing: `measurable` (scenario B4 for rows 7, 9, 10, 11 and 12, and S-0412 scenario K10
      for the level 0 that sends the neutral and special houses' infantry to row 7; row 4 by
      scenario B15 case f; row 6 by scenario B6 case e; scenario B29, case a, for row 2, for row 5 (a3) and for the
      *area guard* exception with a target; S-0309 scenario A6, parts g, n and o, for row 1's
      memory remaining after refresh, and ordinary rest after removal). These parts are `unmeasured`:
      - the *patrol* exception, because the *patrol* processing belongs to other topics (claim 34)
        and may queue a behaviour of its own at the same moment,
      - row 3, the queued destinations, which the order rules fill (S-0213) and for which no input
        is known here,
      - row 1's *wait* case, which differs from the later rows only for a unit with a
        target or a destination (claim 7 already refuses a queued *guard* under *wait*), while no
        setup is known that gives a unit in *wait* a target or a destination at a fall-back to rest,
      - row 8, the harvesting slave of a computer-controlled owner, which is the original's only
        and which only the economy topic's units can be put into.
    - Confidence: high (rows 1 to 6, 11 and 12); medium (rows 7 to 10)

15. **Falling back to rest: vehicles.** When a vehicle falls back to rest, after the common effects of
    claim 13 the result is the first matching row:

    | # | Situation | Result |
    |---|---|---|
    | 1 | it has attack-move memory, or its current behaviour is *wait* | nothing changes |
    | 2 | no destination and non-empty queued destinations | the first queued destination becomes its destination, and the vehicle queues *move* as in row 3 |
    | 3 | it has a destination | queue *move* |
    | 4 | it is a harvester type (ore or weeds) | if it is in contact with another object, or *harvest* is current or queued: nothing changes, and target and destination are kept; otherwise target and destination are cleared and it queues *harvest*, except that in an ordinary fall-back a human-controlled harvester that does not stand on its resource's land type (ore for ore harvesters, weeds for weeders) queues *guard* instead; in the placement fall-back the land type is not looked at |
    | 5 | unarmed, a loaner with passengers, not a team member | queue *unload* |
    | 6 | unarmed and deploying; or unarmed with *unload* current and a deploys-into type; or unarmed with *area guard* current | nothing changes |
    | 7 | unarmed | clear target and destination, queue *guard* |
    | 8 | armed, with *guard* or *area guard* current | nothing changes |
    | 9 | armed, current behaviour has the zombie or the paralysed property | nothing changes |
    | 10 | armed, the owner's house skill level is below the area-guard threshold, and the vehicle has neither the guard-area veterancy ability nor a type that defaults to area guard | queue *guard* |
    | 11 | armed, and a team member, or a harvesting slave, or a slave master | queue *guard* |
    | 12 | armed, otherwise | queue *area guard*, or *guard* if the vehicle sits in a bunker |

    No row queues anything while the current behaviour is *patrol*, *area guard*, *unload* or
    *eaten*, or while it is *enter* towards a bunker it is in contact with and the vehicle does not
    yet stand on that bunker's cell; the clearing of target and destination in rows 4 and 7 still
    happens then. A vehicle left in *unload* with nothing queued stays in *unload*: falling back
    does not itself start a resting behaviour. For a harvester that loses contact while unloading,
    S-0402 claim 11 owns the repeated fall-back and retained load, and
    `quirks.unload_without_contact_stalls` (default original) and its alternative.
    Every "queue" of this table is a queue without an immediate start. Unlike
    infantry, rows 10 to 12 do not look at whether the owner is human-controlled; only the owner's
    house skill level counts. In a skirmish of the original a human-controlled house and the
    neutral and special houses have skill level 0, and the house of every computer player has the
    maximum skill level of the AI settings (S-0412, claim 3). So, whenever the area-guard threshold
    is above 0, an armed vehicle of a human-controlled owner, or of the neutral or special house,
    in skirmish rests in *guard* by row 10 unless it has the guard-area ability or its type
    defaults to area guard. Campaign house skill assignments are not specified here.

    **In our engine** the rows apply to the vehicles of every house as written, with these
    consequences of the houses rule (Terms):
    - row 4's exception for a harvester that does not stand on its resource's land type applies to
      the harvesters of every house, AI houses included,
    - an AI house has the house skill level of a human-controlled house (0 in a skirmish, Terms
      *house skill level*), so its armed vehicles rest as a human player's do,
    - the neutral and special houses keep level 0 (S-0412, claim 3), so their armed vehicles also
      rest by row 10 whenever the area-guard threshold is above 0,
    - no vehicle is a team member, so the team-member parts of rows 5 and 11 never hold.

    **Original only (note for AI designers).** In the original, a harvester of a computer-controlled
    house queues *harvest* at an ordinary fall-back wherever it stands (row 4); the armed vehicles
    of a computer player's house in skirmish rest in *area guard* whenever the maximum skill level
    is at or above the area-guard threshold (rows 10 and 12); and an armed team member rests in
    *guard* (row 11). Our AI can get these results by ordering its harvesters to harvest and its
    resting vehicles to guard an area.
    - Backing: `measurable` (scenario B4 for rows 4, 7, 10, 12, the team-member part of row 11
      and the skirmish skill levels of the human-controlled and computer player houses; S-0412
      scenario K10 for the level 0 of the neutral and special houses; row 3 by scenario B15 case f, row 9 by scenario B6 case e; scenario B29 for row 8, row 12's
      bunker variant and the exceptions for *area guard* and *eaten*; S-0309 scenario A6,
      parts g, n and o, for row 1's memory and rest after removal; S-0402 claim 11 and its
      scenario H12 for a harvester staying in *unload* without contact).
      These parts are `unmeasured`:
      - row 2, the queued destinations: no input here is known to fill that list (as for
        claim 14, row 3),
      - row 1's *wait* case: no setup is known that makes a waiting vehicle fall back to rest
        with a destination or another state that would distinguish row 1 from the later rows.
        Queueing *guard* is already refused under *wait* (claim 7),
      - row 4's case of a harvester in contact with another object or with *harvest* current or
        queued: the processings that would make such a harvester fall back to rest belong to the
        economy topic's harvest cycle (S-0402), so no setup here can time a fall-back in that state,
      - row 5, because which actors are loaners is not established (Open questions) and so no
        setup is known that has a loaner with passengers fall back to rest,
      - row 6: a vehicle that is deploying, or in *unload* with a deploys-into type, falls back
        to rest only through the deploy and transport specs' rules, which no setup here can time,
        and the case of *area guard* has the same outcome as row 7 under the *area guard* exception
        (nothing queued) and cannot be told apart from it,
      - the slave-master and harvesting-slave parts of row 11, which concern the economy topic's
        units,
      - the exceptions for *patrol*, non-harvester *unload* and *enter* towards a bunker: the *patrol* and
        non-harvester *unload* processings belong to other topics (claim 34), which may queue a
        behaviour of their own at the same moment, and the *enter* case lasts only while the
        vehicle drives onto the bunker's cell, which no setup here can time a fall-back into,
    - Confidence: medium (the rows and the skill level of houses); low (the loaner row)

### Behaviour properties

16. **No-threat.** While an actor's current behaviour has the no-threat property, no other actor's
    automatic search picks it as a target. A mobile actor that is not a team member falls back to
    rest at once (claims 13 to 15; S-0501 claim 18 for an aircraft) after a hit that names an
    attacker (S-0302 claim 1), lowers its health and reports *unchanged*, *now yellow* or *now red*
    (S-0302 claim 12), when the behaviour that is current at that point of the hit (below) has the
    no-threat property and not the zombie property. The fall-back normally ends the no-threat
    behaviour.
    - **Results.** A hit that reports *unaffected* never triggers it. So a healing hit (S-0302
      claim 11), which raises health and reports *unaffected*, does not, and neither does any
      other hit that reports *unaffected*, such as one that changes no health. A hit that reports
      *now dead* or *post-mortem* does not trigger it either. A berserk hit, which lowers no health,
      is open here (Open questions).
    - **Order within the hit.** The test comes after the hit's change of health (S-0302), the
      anger change (S-0309 claim 16), the retaliation (claim 18) and the scatter from damage
      (claim 19, and request A of S-0208 claim 15 for infantry). It comes before the actor's other
      reactions to the hit: for an infantry unit, request B of S-0208 claim 15 and the change of
      fear (S-0107 claim 16), and for a vehicle the crush response (claim 39). An aircraft has no
      reaction to the hit after it. So:
      - an actor that retaliates at that hit has *attack* current (claim 10) when the test is made.
        It does not fall back unless *attack* has the no-threat property, and its no-threat
        behaviour stays suspended until the revert. An actor that does not retaliate at that hit is
        tested on the behaviour that the scatter from damage leaves current. For an aircraft that is
        sent the scatter from damage (claim 19) while its current behaviour has the scatter
        property, that request has already made it fall back to rest (S-0501 claims 49 and 18), so
        the test is made on the behaviour that this first fall-back leaves current. For any other
        actor it is the behaviour it had when the hit landed,
      - the scatter from damage of claim 19 (request A for infantry) is decided on the target and
        destination that the actor had before the fall-back, and request B on the state that the
        fall-back leaves,
      - the anger and fear changes depend on nothing that the fall-back changes,
      - the crush response's *move* replaces whatever the fall-back queued (claim 39).
    - Backing: `measurable` (scenario B5: part a for the search exclusion and the fall-back after a
      hit that lowers health, part b for the zombie property, part c for the order against
      retaliation, part d for a healing hit; claim 39's own backing for the crush response). The
      other *unaffected* hits, the *now dead* and *post-mortem* results, the team exception and the
      order against the scatter requests are `unmeasured`: no scenario here lands another
      *unaffected* hit from a known attacker on an actor in a no-threat behaviour, puts a team
      member in one, or gives a no-threat infantry unit a guard spot on another cell or makes it a
      fraidycat, and an actor that a hit kills has no behaviour left to observe. Part a, whose
      subject does not retaliate and is not an aircraft, backs the test on the behaviour it had when
      the hit landed. For an aircraft sent that request while its behaviour has the scatter
      property, the test on the behaviour that its first fall-back leaves is `unmeasured`: no
      scenario here hits an aircraft in a no-threat behaviour. The order against anger and fear, and
      the aircraft's lack of a later reaction, are `unmeasured` as well: they change nothing that a
      run can record.
    - Confidence: high (the search exclusion, the fall-back after a hit that lowers health, the
      zombie exception, the order against retaliation, the test on the behaviour at the hit for an
      actor that does not retaliate and is not an aircraft, and the healing hit); medium (the test
      for an aircraft sent the scatter from damage while its behaviour has the scatter property, the
      other *unaffected* hits, the *now dead* and *post-mortem* results, the team exception, the
      order against the scatter requests, anger and fear, and the aircraft's lack of a later
      reaction), and the crush response's place after it has claim 39's confidence

17. **Zombie and paralysed.** At a fall-back to rest, the zombie or the paralysed property of the
    current behaviour matters only through its row in the tables of claims 14 and 15 (row 6 for
    infantry, row 9 for vehicles): when that row is the first that matches, nothing is queued and
    the behaviour stays current. The rows before it take precedence, so in these cases those rows
    decide, whatever the property says:
    - an infantry unit that has a target (row 2 queues *attack*), or a destination or queued
      destinations (rows 3 and 4 queue *move*),
    - a vehicle that has a destination or queued destinations (rows 2 and 3 queue *move*), a
      harvester (row 4), and an unarmed vehicle (rows 5 to 7; row 9 is for armed vehicles only, so
      an unarmed vehicle in such a behaviour queues *guard* by row 7 unless row 5 or 6 applies).

    A behaviour that those rows queue becomes current at the actor's next per-tick start at which
    it is ready to change (claim 11). Rows 1 and 5 of claim 14, rows 1 and 8 of claim 15 and the
    behaviours for which those tables queue nothing keep the current behaviour whatever its
    properties. Orders and other rules can replace a zombie or paralysed behaviour through the
    queue or by force, subject to readiness (claim 11). In addition:
    - a vehicle with the paralysed property refuses every scatter request that has a threat point,
      forced or not. A request without a threat point is not checked this way, and infantry and
      aircraft have no such check. The request that damage sends to a vehicle never has a threat
      point (claim 19), so the paralysed property never stops a vehicle's scatter from damage. The
      threat-point request of S-0208 claim 15 (request B) is sent to infantry only. Other rules
      can send a vehicle a request with a threat point. For example, once a unit that is entering
      a bunker has stopped on the bunker's cell, the unit nearest the centre of each of the
      bunker's cells, other than the entering unit, is sent a forced request, not move-anywhere,
      with the bunker's position as threat point. S-0606 claim 6 owns this clearing rule,
      including exclusion of the bunker itself.
      Factory apron-neighbour clearing is another sender: S-0417 claim 4 sends forced,
      move-anywhere requests with the apron centre as threat point, both while unloading and
      under that spec's idle clearing rule (claim 13),
    - the no-threat fall-back of claim 16 does not happen while the zombie property holds.
    - Backing: `measurable` (scenario B6: part e for the fall-back rows, with an infantry unit and
      an armed vehicle reaching row 6 or 9 and an unarmed vehicle reaching row 7 first; part b for
      the damage scatter of a paralysed vehicle; part c for the refusal of a request with a threat
      point, including its factory variant; S-0606 claim 6, scenario K1, for bunker clearing,
      and scenario B5, case b, for the zombie property
      suppressing the no-threat fall-back). The
      precedence of a target, a destination or queued destinations over the zombie and paralysed
      rows is `unmeasured`: *attack* falls back to rest only without a target and *move* only
      without a destination, the fall-back of *capture*, *sabotage* and *eaten* (claim 28) happens
      with a target only for an actor without a destination whose target is not a building it can
      take, which no known order or rule sets up in those behaviours, and no other setup is known that makes an actor in a zombie or paralysed
      behaviour fall back to rest while it holds a target or a destination. It is the row order of
      claims 14 and 15, which states it.
    - Confidence: high for the fall-back rows, their order, and the request from damage having no
      threat point; medium for the refusal of a request with a threat point and the bunker's
      nearest-object clearing rule (S-0606 claim 6)

18. **Retaliate.** An actor retaliates against an attacker only while its current behaviour has the
    retaliate property, among the other retaliation conditions that the target-acquisition specs
    own. Three of those conditions concern behaviours and are stated here:
    - a mobile actor of a human-controlled owner retaliates only while its current behaviour is
      *guard*, *area guard* or *patrol* (not *sticky*), unless the game-wide "player return fire"
      setting is on (Import mapping); buildings are not restricted this way,
    - an actor of a human-controlled owner that already has a target never retaliates,
    - retaliation happens as an override to *attack* with the attacker as target (claim 10). An actor
      of a computer-controlled owner, or one whose weapon already reaches the attacker, overrides at
      once; an actor of a human-controlled owner that cannot reach the attacker overrides only if
      its 3-D distance to the attacker is at most (*S* + 0.5) × 256 leptons, where *S* is its
      type's sight in cells (Formulas F7).

    **In our engine** the three conditions for human-controlled owners apply to the actors of every
    player house, human or AI (Terms, houses in our engine). The neutral and special houses keep
    the original's rule for owners that are not human-controlled, stated next, and their other
    retaliation conditions are those of S-0309 (claim 25, part h). **Original only for player
    houses (note for AI designers):** in the original, none of the three conditions holds back a
    mobile actor of a computer-controlled house: the human owner's behaviour list, existing-target
    prohibition and distance limit do not apply, and it overrides at once when retaliation is
    allowed. An existing actor target does impose a different condition: the attacker's threat
    sum must be at least the current target's, using S-0309 claim 25 part h's comparison, not
    the converted integer threat score. The other retaliation conditions still apply, including
    the retaliate property above and S-0309 claim 25 part h's prohibition while draining.
    - Backing: `measurable` (scenario B7: cases a to d for the behaviour list, the player
      return-fire setting and the distance rule, case g for an actor that already has a target,
      and its computer-controlled runs for the original-only part, including the threat-sum
      comparison; S-0309 claim 25 part h for that comparison's backing and confidence)
    - Confidence: high (the property and the behaviour list); medium (the distance rule and the
      rule about an existing target)

19. **Scatter.** The scatter property acts on scatter requests in two ways. It decides whether
    some requests are accepted, and the scatter from damage below tests it before it sends one.
    Acceptance depends on the kind of actor:
    - A vehicle refuses a request that is not forced unless its current behaviour has the scatter
      property. The property does not decide about a forced request. The vehicle's other
      acceptance conditions, and where it then goes, are S-0207 claim 16. Among them, a vehicle
      whose current behaviour (or, with none current, its queued behaviour) is *sleep*, *sticky* or
      *unload* refuses every request, forced or not.
    - For an infantry unit, S-0208 claim 11 gives the whole acceptance rule. There a forced request
      counts as not forced while the unit's locomotor is moving, so the property can refuse it then.
    - An aircraft falls back to rest when its current behaviour has the scatter property, or
      when it has no current behaviour (claim 1's exception: whatever is queued), and otherwise
      ignores the request, whatever its options (S-0501 claim 49). The forced option does not
      decide for an aircraft either.

    **Scatter from damage.** The scatter from damage of an infantry unit is owned by S-0208
    (claim 15, request A, with the acceptance and destination in its claims 11 to 13); this
    claim does not restate it. For every other mobile actor (a vehicle or an aircraft), when it
    takes damage and survives it, it is sent one scatter request when either of these cases
    holds. The request has no threat point, is forced and is not move-anywhere.
    - *Case 1.* The damage has a known attacker, and the actor may retaliate against it (claim 18)
      but does not retaliate against this hit. It has neither a target nor a destination, and the
      player-scatter setting (Import mapping) is on or the actor has the scatter veterancy ability.
      Its owner and its current behaviour do not matter. An example is a human-controlled actor
      beyond the reach of F7. Retaliating gives the actor a target (claims 10 and 18), so an actor
      that retaliates is never sent the request this way.
    - *Case 2.* The damage has no known attacker or the actor may not retaliate against it, and all
      of these hold:
      - the actor is not an aircraft,
      - its current behaviour has the scatter property,
      - it is not held by a tether link,
      - its locomotor is not moving (S-0208 Terms),
      - it has neither a target nor a destination,
      - its owner is computer-controlled, or the player-scatter setting is on, or the actor has the
        scatter veterancy ability.

    An infantry unit can also be sent request B of S-0208 claim 15, which has the attacker's
    position as threat point and is not forced. Damage sends no other actor a request with a
    threat point. The request of both cases is forced, so the scatter property never makes a
    vehicle refuse it once sent, though the vehicle's other refusals (S-0207 claim 16) still apply.
    In case 2 the property is tested before the request is sent, so
    an actor whose behaviour lacks it is not sent the request in that case.

    **In our engine** (non-infantry actors; S-0208 claim 15 for infantry) the last condition of case 2 reads "the player-scatter setting is on, or the
    actor has the scatter veterancy ability" for the actors of every house (Terms, houses in our
    engine). **Original only (note for AI designers):** in the original, the mobile actors of a
    computer-controlled house meet that condition always, so they are sent the request of case 2
    whatever the player-scatter setting says (whether they accept it is decided as for any
    other actor).
    - Backing: `measurable` (scenario B6 for vehicles, case 2 in part b, with its variant for the
      original's computer-controlled owners, and case 1 in part d; S-0208 scenario W7 for
      infantry). Case 1 for aircraft, apart from the answer that the next sentences back, is
      `unmeasured` here: B6 tests driving vehicles only, and no scenario establishes the
      movement that follows an aircraft's acceptance of this request.
      The aircraft's answer to a scatter request has S-0501 claim 49's backing. It is
      `measurable` (S-0501 scenario AC31 part e) only for the scatter from damage and a cell's
      request, sent without the move-anywhere option to a landed aircraft of a human-controlled
      owner that is armed, has ammunition left and has no contact. Every other case is
      `unmeasured`, as S-0501 claim 49 rates it: the move-anywhere option, both requests in pad
      contact, the request of claim 28 (as claim 28 rates it), and an aircraft with no current
      behaviour, since no setup is known that sends an aircraft a request between its creation
      and its first turn.
    - Confidence: medium (the aircraft's answer, as S-0501 claim 49 rates it); high (the gate on
      requests that are not forced, for vehicles; the options of the
      request from damage); medium (the conditions of the two cases, for non-infantry actors)

20. **Recruitable.** The recruitable property is imported for every behaviour (Import mapping), and
    in our engine it has no effect: no rule of any house reads it (Terms, houses in our engine).
    **Original only (note for AI designers):** in the original the property affects only the
    computer player's choice of which actors to send to defend its base when the base is attacked:
    an actor whose current behaviour lacks the property is never picked for that defence.
    Our AI may honour it when it picks units for such tasks. Team recruitment is outside
    this spec's scope; no restriction on the original's team selection is asserted here.
    - Backing: `measurable` (scenario B20, for the original's base defence)
    - Confidence: medium

21. **The anti-air rate is used by buildings only.** The anti-air rate replaces the rate in the
    delay of an armed building's *guard* processing, which also serves its *sticky* and *area guard*
    (each reading its own behaviour's anti-air rate), and in one rare case of the *attack*
    processing of a building whose type has the SAM flag (claim 32, out-of-stage rule). No other processing of
    any actor kind uses it. The anti-air rate is reset on override (glossary): a rules layer
    that contains a behaviour's section but gives no anti-air rate, or gives 0, sets the
    anti-air rate to the rate as it stands after that layer
    (`quirks.anti_air_rate_layer_reset`, Import mapping).
    - Backing: `measurable` (scenarios B8 and B13; scenario B16, case m, for the SAM case, which
      only a test building type reaches, claim 32).
    - Confidence: high

### What the general-purpose behaviours do

The claims below describe the processing of each behaviour for mobile actors unless they say
otherwise. "Infantry", "vehicles" and "aircraft" mark rules for one kind only. Rules, rows and
conditions that apply only to computer-controlled owners are marked **original only**: they
describe the original's computer player, as notes for the AI designers, and our engine applies
none of them to the player houses (Terms, houses in our engine). Rules for human-controlled owners
apply in our engine to the actors of every player house.

22. **Move.** If the actor has no destination, is not moving and has nothing queued, it falls back
    to rest and chooses a delay of 1 tick. Otherwise it chooses the standard delay. The movement
    itself is carried out by the locomotion rules on every tick, not by the processing.

    **End-of-update check (infantry).** At the end of each update of an infantry unit, after its
    late start (claim 11), the unit is checked: if its current behaviour is *move*, or it has no
    current behaviour and *move* is queued, and it is not moving (S-0208 Terms, the same test as in
    the general rule above) and has no destination, it falls back to rest (claims 13 and 14). This
    check is separate from the processing. It is made whether or not *move* was processed in that
    update and whatever is queued (medium, `unmeasured` for an update with no *move* processing or
    with something queued, Backing). A behaviour that the fall-back queues without an immediate
    start is not started in that update, because the late start has passed. It becomes current at
    the unit's next per-tick start at which the unit is ready (claim 11), the first start of the
    next tick if it is ready there. The start restarts the processing timer (claim 8), whatever
    delay the last *move* processing chose: at a first start the behaviour is processed there at
    once (claim 3 (a)), and at a later start, such as a late start, on the next tick or in a second
    turn of the same tick (claim 3 (b), claim 2). So the check catches an infantry unit whose *move*
    is still current at the end of an update after which it has neither a destination nor movement
    in progress. This happens in the update whose locomotion clears its destination, by an arrival
    that ends its walk (S-0208) or a search that fails at the reachability gate (S-0212 claim 23),
    after a stop order (claim 12) in the update whose locomotion ends the step under way, and in the
    jump tick of a teleporting infantry unit, whose jump in its own locomotion stops its movement
    and removes its destination (S-0207 claim 20 (e)), so that its rest behaviour is queued at the
    end of that tick. A *move* processing that finds the unit already stopped, with no destination
    and nothing queued, falls back to rest by the general rule above instead, and its rest behaviour
    can become current at that update's late start, before the check. What the fall-back leaves is
    given by claim 13's common effects and then claim 14's first matching row: a route holder issues
    its next point after a queue of *wait* with an immediate start, so a holder that is ready to
    change (claim 11) has *wait* current at the end of that update and queues no *guard* (claim 13,
    S-0213 claim 16c), a guard spot on another cell gives the unit a new destination by claim 13,
    and a target, queued destinations or attack-move memory decide by claim 14's earlier rows. For a
    unit with none of these, whose *move* has neither the zombie nor the paralysed property, the
    fall-back queues its rest behaviour: it ends that update with *move* current and its rest
    behaviour queued, and, if it is ready there, has its rest behaviour current at the first start
    of the next tick and processed there. When the update's own locomotion clears the destination, a
    *move* processing earlier in that update sees the destination as it was before the locomotion
    (S-0203, from S-0516 claim 42: ordinary mobile locomotion follows due behaviour processing), so
    it chooses the standard delay and takes its draw (medium, `unmeasured`, Backing). For the gate
    failure this holds alike for a unit in its first turn after its release from a factory and for a
    unit that was resting before the order. An update that ends before its late start (claim 11) has
    no check. A team member's current behaviour under a team script's move step was not traced, so
    whether the check catches it is open (Open questions); our engine has no team members (claim
    14).
    - Backing: `measured` (M-0023 run 1: at the end of the turn whose search fails at the gate,
      *move* current, *guard* queued and no destination; *guard* current in the next tick) for a
      released infantry unit in its first turn, and (M-0023 controls: a walker given a destination
      in another reach region at the start of a single step arrives, its destination is cleared in
      the next tick with *move* current and *guard* queued, and *guard* is current in the tick after
      that) for the failed gate in the tick after an arrival. An arrival that ends the walk at its
      own destination is not in M-0023: its controls' destination lies in another reach region, so
      their arrival does not end the walk. `measurable` (scenario B49) for a resting infantry unit
      on open ground, after a failed gate (part a), an arrival (part b), a stop order (part c) and a
      teleporting unit's jump (part d). The standard delay of the *move* processing in that update
      is `unmeasured`: M-0023 records in that tick one draw that its baseline does not have, and the
      recording ties no draw to a unit, and *move* is replaced before that delay ends. The check
      made whatever is queued, with *move* queued and nothing current, or in an update with no
      *move* processing, and no check in an update that ends before its late start, are
      `unmeasured`: no scenario here sets up a queued behaviour or such an update at that point, and
      claim 11 owns the updates that end early. What the fall-back leaves has the backing of the
      claim 13 and claim 14 rows that decide it.
    - Confidence: high for the released unit's slots at the end of that update and for its rest
      behaviour being current in the next tick, and for the walker's slots after a failed gate in
      the tick after its arrival (M-0023 controls). Medium for the resting unit, for arrival, the
      stop order and the teleporting unit's jump tick, for the check with *move* queued and nothing
      current, for the check being made whatever is queued and whether or not *move* was processed,
      for the standard delay in that update and for no check in an update that ends before its late
      start. What the fall-back leaves has the confidence of the claim 13 and claim 14 rows that
      decide it.

    **Vehicle goal cleanup (Q-0797).** An ordinary driving or ship vehicle's *move*
    processing does not remove a retained destination merely because the vehicle occupies
    its goal cell. S-0207 claim 26 owns the separate locomotion cleanup: a cell goal is
    removed at a qualifying no-track update, not by final-track completion alone. Thus a
    same-cell move does not remain a moving destination indefinitely. Once that cleanup
    leaves no destination and the other conditions above hold, the next due *move*
    processing falls back to rest and chooses 1 tick. Cleanup does not itself expire the
    processing timer, so the fallback need not be in the cleanup tick or the next tick.
    A due processing before cleanup still sees the retained destination and uses the
    standard delay. This does not add a cleanup wait to locomotion.

    S-0207 claim 26 also owns destinations naming a vehicle, infantry unit or aircraft:
    no-track equality with the cell of the supplied movement point alone clears neither
    destination, whether the named unit is moving or stationary. Its separate exception
    applies when the mover rests in *guard* and its world point exactly equals the retained
    locomotor destination point in all three integer-lepton coordinates, including height,
    without rounding. That exception removes the goal or invokes the approach-destination
    fallback specified there; it does not compare the named unit's current position.
    Behaviour-issued replacement destinations remain separate effects.
    - **Vehicles:** a deployed vehicle or one in the middle of deploying or undeploying instead
      queues *guard* and chooses 1 tick (S-0517 claim 10; transitions: S-0517 claims 3 to 8).
      This special fallback applies only to *move*, not to *patrol*.
      Otherwise the unfinished unload or deploy cycle would restart without changing the
      general outcome or delay. No rule starts that cycle for vehicles, so it is always
      finished and neither that restart nor a readiness delay from it occurs (S-0517 claim 15).
    - **Infantry:** a deployed infantry unit (current sequence one of the deploy kinds) of a
      human-controlled owner instead clears its destination and chooses 1 tick; one of a
      computer-controlled owner whose type's undeploy delay is negative starts its *undeploy*
      sequence and chooses that sequence's frame count as the delay. A deployed unit of a
      computer-controlled owner whose undeploy delay is 0 or more follows the general rule. In our
      engine the rule for human-controlled owners applies to the deployed infantry of every house; the
      two rules for computer-controlled owners are the original's only (note for AI designers:
      in *move*, the original's deployed computer infantry packs up by itself if its type has a
      negative undeploy delay, and otherwise follows the general rule above).
    - Backing: `measurable` (scenario B3; scenario B25 for deployed infantry; S-0517 P3 b for
      deployed vehicles and P6 for the inactive cycle; B43 and S-0207 D30 for goal
      cleanup versus the next due processing; S-0207 D31 for unit-goal retention and
      the exact-point guard exception, inheriting claim 26's measurement limitations).
      Processing *move* during a vehicle's
      deploy or undeploy transition is `unmeasured`: ordinary orders cannot start it until
      the transition ends (S-0517 claim 10).
    - Confidence: high (the general rule and deployed-vehicle fallback); medium (the infantry
      special cases and inactive cycle); low (processing during a vehicle transition)

23. **Attack.** The general rules of an *attack* processing are the four below. Every rule whose
    condition holds applies in the same processing:

    | Rule | Applies when | Effect |
    |---|---|---|
    | hover-attack cell | the actor's type is a hover-attack type and its height above the surface is 0, with or without a target | it takes a nearby cell as its destination |
    | charging break-off search | the actor is infantry with its charging mark set, and processing reaches these general rules | it runs S-0518 claim 15's automatic range search; any nonempty result becomes its target and clears the mark, even if equal to the held target; an empty result leaves target and mark unchanged |
    | approach | the actor has a target | it takes the approach step of claim 37 (engaging the target is the combat specs') |
    | fall-back | the actor has no target | it falls back to rest |

    "Has a target" and "has no target" are judged on the target the actor has once the
    charging break-off search has taken effect, so a nonempty result is approached in the same
    processing. The mark, not the held target's ownership or usability, gates the search.
    S-0518 claim 15 owns candidate eligibility and ranking, the bypass of target-retention,
    passive-acquisition and opportunity gates, and the absence of a targeting-timer gate or
    reset draw. A nonempty result is accepted without another held-target ownership,
    firing-legality or identity check. Vehicles do not run this break-off search.

    The delay of these general rules is the standard delay, *D* = *D*<sub>r</sub> + *U*(0, 2),
    with the draw taken whether or not the halving below applies. It is halved, trunc(*D* / 2),
    when the actor has a target, either it is an infantry unit whose type has the close-range flag
    or its primary weapon's range is at most 512 leptons, and its 2-D distance to the target is
    from 282 to 768 leptons (Formulas F3, F4).
    - **Infantry:** the rules of this table take precedence over the general rules above. When a
      row applies (the first that does), it decides the outcome and the delay, and none of the
      general rules happens (no hover-attack cell, no charging break-off search, no approach, no
      halving):

      | Situation | Result | Delay | Draws |
      |---|---|---|---|
      | it has explosive charges (by type or veterancy ability), and its target is a building that accepts charges and is not invisible | takes the target as destination and queues *sabotage* | 1 tick | 0 |
      | **original only:** computer-controlled owner, its target is a building, and its type infiltrates, or occupies or assaults buildings and may occupy this one | takes the target as destination and forces *capture* | 1 tick | 0 |
      | human-controlled owner, and it is deployed (current sequence one of the deploy kinds) | runs its deployed-firing rule (S-0517 claim 16) | the standard delay, not halved | 1 |

      The first row is not limited to computer-controlled owners. **In our engine** the second row
      never matches, and the third applies to the deployed infantry of every house. **Original
      only (note for AI designers):** an infiltrator, occupier or assaulter of the original's
      computer player that is attacking a building it may take goes straight for it, in *capture*; our
      AI can get the same result with a capture or occupy order (S-0213, S-0214).
    - Backing: `measurable` (scenario B9; cases c and d for the second and third infantry
      rows; scenario B24, case d for the hover-attack cell and case c, with S-0518 scenarios
      TC3 and TC4, for the mark-gated charging break-off search and replacement).
      The explosive-charges row is `unmeasured`: S-0214 translates a player's attack order on
      a building that accepts charges into *sabotage*, and S-0309 claim 18 prevents such a
      human-controlled unit from retaliating against a building. B9 (b)'s former retaliation
      setup therefore cannot reach *attack* with that target. No replacement input that isolates
      this row is established here.
    - Confidence: high (the delay rule, mark-gated break-off search and replacement, and that
      the 1-tick rows replace the general rules and take no draw); medium (the conditions of the infantry rows)

24. **Guard.** The actor stays where it is. The kind-specific rules at the end of this claim
    (deploying infantry, vehicles) take precedence over the general rules below.

    **Enter-nearest instruction.** In our engine only a unit of a neutral or special house can hold
    one, and only when the bio-reactor fate of S-0605 claim 16 has given it the absorbing-building
    instruction. The search made here at each processing, and the rule that ends the instruction when
    it finds no building, are this claim's (S-0605 claim 16 says so); S-0605 claim 16 gives the fate's
    first search and its candidates. The two occupy instructions are the original's only (note for the
    AI designers). Under the general rules, a processing of an actor that holds an
    enter-nearest instruction (Terms) carries out that instruction and nothing else of the general
    rules, the delay table below included. The actor looks again for the nearest suitable building
    by the 3-D distance of F4: for *enter the nearest absorbing building*, the absorbing buildings
    of its own owner that accept it as a passenger; for *occupy the nearest owned occupiable
    building*, its owner's owned occupiable buildings that it may occupy; for *occupy the nearest
    occupiable building*, every building of any owner that it may occupy. If it finds one, it takes
    that building as its destination and queues *enter* (absorbing building), or queues *capture*
    with an immediate start (the two occupy instructions), unless it is already bound for that
    building (for *enter*: in contact with it, or waiting in line for it (S-0605 claims 3 and 16); for *capture*: with that building as destination and
    *capture* current). If it finds none, the instruction lapses. Either way it chooses
    *D*<sub>r</sub> with no draw, taken from the rate of the behaviour that is current once this
    has taken effect (claim 6): *capture*'s when the immediate start made *capture* current,
    otherwise the rate of the behaviour being processed (*guard* or *sticky*). Our AI can get the
    same result by ordering such a unit to enter or occupy the building it picks (S-0213, S-0214).

    **Other actors.** Under the general rules, an actor that holds no enter-nearest instruction has
    the effect of this table at each processing, chosen by whether it had a target when the
    processing began:

    | Target when the processing began | Effect |
    |---|---|
    | none | an infantry unit or vehicle with an electric-charge secondary weapon runs S-0518 claim 14's automatic charging scan; a found building becomes its target, its charging mark is set and *attack* is queued without an immediate start; otherwise ordinary guard processing runs, including the infantry idle action of S-0107 claim 14 |
    | present | a hover-attack type whose height above the surface is 0 takes a nearby cell as its destination; firing at a target in range is done by the firing rules, not by the processing |

    A successful charge scan replaces ordinary guard processing in that processing, not the
    delay selection below. S-0518 claim 14 owns the eight-neighbour scan in step-direction order
    and the own-house-only building eligibility. An actor that makes a charge target does not
    attempt its ordinary idle action or take the hover-attack cell in that processing. **Original only (note for AI designers):** in addition, an infantry unit of a
    computer-controlled owner with explosive charges whose target, as this table leaves it, is a
    building queues *sabotage* (unless that is already current). Our engine applies this to no
    house; our AI can give such a unit a sabotage order instead.

    The delay is that of the first row that matches:

    | Situation | Delay | Draws |
    |---|---|---|
    | the actor's weapon is still rearming | the rearm time remaining | 0 |
    | the type has the distributed-fire flag and the actor's distributed-fire candidates (Terms) are not empty | 0 | 0 |
    | any other case | the standard delay | 1 |

    Apart from automatic charging, *guard* takes no target by itself in its processing. Its
    ordinary automatic enemy search runs on the targeting timer (claim 33).
    - **Infantry** of a type that deploys: the deployer rules of claim 31 take precedence. When one
      of them applies, it decides the outcome and the delay, and nothing else of this claim happens.
    - **Vehicles:** the rules of this table, which belong to other topics, take precedence over the
      general rules, and earlier rows over later ones. A row applies when its condition holds and
      no earlier row marked "yes" in the last column applies. When a row marked "yes" applies, its
      result and delay are the whole outcome: no later row and none of the general rules apply.
      Row 3, marked "no", replaces nothing: its effects come in addition to those of whatever
      applies after it (row 4 or the general rules).

      | # | Vehicle and condition | Result | Delay | Excludes later rows and the general rules |
      |---|---|---|---|---|
      | 1 | a slave master for which the wake-up rule of S-0402 (claim 14) holds | wakes its slaves | the standard delay | yes |
      | 2 | a harvester or weeder, in the cases of the guard rules of S-0402 that queue *harvest* again (claim 13, for an owner that is not human-controlled; claim 17, the chrono miner) | queues *harvest* as those rules say | 1 tick, no draw | yes |
      | 3 | a weeder, in the cases of the weeder's guard rule that do not end the processing | as that rule says (it can queue *harvest*) | none of its own | no |
      | 4 | **original only:** a vehicle that deploys into a construction yard and belongs to a computer-controlled owner whose automatic base building is on | queues *unload* (to deploy) | the standard delay, with the rate of the behaviour being processed | yes |

      No row applies to a slave master whose wake-up rule does not hold, unless another row's
      condition holds for it. A harvester for which no guard rule of S-0402 queues *harvest* again
      (for example one of a human-controlled owner that is not a chrono miner, S-0402 claim 13)
      matches none of rows 2 and 3 and goes on to the general rules. Row 3 is the weeder's rule of
      the original, which S-0402 leaves to a later revision (S-0402, Edge cases, weeders); this
      spec states only that it adds to the general rules and has no delay of its own. **In our
      engine** row 2's case of S-0402 claim 13 applies only to the harvesters of the neutral and
      special houses, since no player house is computer-controlled (S-0402, claim 13), and its
      case of claim 17 to the chrono miners of every player house (S-0402, claim 17). In our engine row 4 never applies; the original's computer player
      uses it to deploy its base-building vehicles by themselves, which our AI does with a deploy
      order.
    - Backing: `measurable` (scenario B10, scenario B21 parts a and b for the occupy instructions,
      scenario B24, case a for the distributed-fire row and case b for the charge target,
      supplemented by S-0518 claim 14's scenarios TC2 and TC4; scenario
      B26, case a for the original-only sabotage rule).
      The absorbing-building instruction's repeat search, lapse, queue-only outcome and
      already-bound exceptions are `unmeasured` here: B21 c leaves the instruction's input
      path conditional, and B21 d supplies no established way to retain the required
      waiting-line state at guard processing. Those parts are investigations only.
      S-0605 claim 16 retains ownership of the fate's initial search and its backing.
      The vehicle table's row 1 is `measurable`
      by S-0402 scenario H7 (the slave miner in *guard* after a stop order; its variants b to e for
      the parts of the wake-up rule), and row 2 by S-0402 scenario H5, parts C and D (the
      computer-controlled harvester's return to *harvest* and the no-field record), and scenario H8
      (the chrono miner in *guard*, its part g for the "full and still moving" case), which see the
      harvester back in *harvest* within a few ticks of its *guard* processing. Rows 3 and 4 are
      `unmeasured`: row 3 is the weeder's rule, whose conditions S-0402 leaves to a later revision,
      so no setup here can tell when it applies; row 4 is the original's only and needs a computer player that deploys its
      base-building vehicles by other means in the same game. The attempt at the idle action is backed by S-0107
      (claim 14).
    - Confidence: high (the delay rule, no target search in the processing); medium (the special
      cases, the enter-nearest search and its delay, and the distributed-fire case, whose list the
      target-acquisition specs own); low (the "already bound" exceptions of the enter-nearest rule)

25. **Sticky.** *Sticky* is processed as *guard* (claim 5), with the properties of *sticky*. It
    differs from *guard* in ways that follow from other claims, whatever the data says:
    - an infantry unit, vehicle or aircraft whose current behaviour is *sticky* is never ready to
      change (claim 11), so queued behaviours, including orders, never start while it lasts and
      only a force or an override replaces it,
    - *sticky* is not among the behaviours that run the per-tick opportunity search (claim 33), and
      the processing of *guard* takes no hostile target by itself (claim 24),
    - a mobile actor of a human-controlled owner in *sticky* does not retaliate unless the
      player-return-fire setting is on (claim 18).

    Observable consequence: a mobile actor of a human-controlled owner in *sticky* never takes a
    hostile target by itself (unless that setting is on); automatic charging under claim 24 is
    still possible. In our engine that hostile-target rule holds for the actors of every
    house. In the original, one of a computer-controlled owner takes a target only by retaliating
    (claim 18, original only). Further differences come only from the imported properties of
    *sticky*: if they mark it zombie or paralysed, a fall-back to rest that reaches the row for
    those properties keeps it current (claim 17). Since an actor in *sticky* is never ready to
    change, a behaviour that a fall-back queues does not start while *sticky* lasts in any case.
    - Backing: `measurable` (scenarios B6 and B17)
    - Confidence: high (readiness); medium (the target consequence)

26. **Area guard.** The actor guards an area around its guard spot. At each processing, the first
    row of this table whose condition holds applies and ends the processing with that row's delay:

    | Row | Condition | Result | Delay |
    |---|---|---|---|
    | 1 | **vehicles:** the vehicle is a slave master and the wake-up rule of S-0402 (claim 14) holds | wakes its slaves | standard delay |
    | 2 | the actor is in a bunker | queues *guard* with an immediate start | 1 tick |
    | 3 | the actor holds an enter-nearest instruction (Terms; in our engine only the absorbing-building one of a neutral or special house unit) | carries it out as in *guard* (claim 24) | *D*<sub>r</sub>, no draw, with the rate of the behaviour then current: *capture*'s when an occupy instruction's immediate start made it current, else *area guard*'s |
    | 4 | **vehicles:** the vehicle is a harvester | queues *harvest* and starts it at once, whether or not it is ready to change | 1 + *U*(1, 10) ticks |
    | 5 | **original only:** an infantry unit of a computer-controlled owner with explosive charges, whose target is a building, while *sabotage* is not current | queues *sabotage* | 1 tick |

    Each outcome of the processing comes with these further effects, and with no others:

    | Outcome | Slave master wakes its slaves anyway | Takes own cell as guard spot | Computer-owner effects |
    |---|---|---|---|
    | a deployer rule of claim 31 applies (Infantry, below) | no | no | no |
    | row 1, 2 or 3 applies | no | no | no |
    | row 4 applies | no | no | yes |
    | row 5 applies | yes | yes | yes |
    | none of rows 1 to 5 applies | yes | yes | yes |

    - *Slave master wakes its slaves anyway:* a slave master wakes its slaves whenever its slave
      group is at rest (S-0402, claim 14), whatever the rest of the wake-up rule says: the kick
      delay, the miner's cell and the short scan are not looked at. A slave master that is also a
      harvester (row 4) never gets this effect, since one of rows 1 to 4 ends every processing of
      it.
    - *Takes own cell as guard spot:* an actor that has no guard spot and nothing queued (not
      counting what row 5 queues in the same processing) takes its own cell as its guard spot. So
      an actor in *area guard* lacks a guard spot after such a processing only if a behaviour is
      queued.
    - *Computer-owner effects* (original only: computer-controlled owners): a unit that has no destination
      and whose first queued destination is its guard spot takes the guard spot as its destination
      and removes that entry from its queued destinations (a looping route puts it back at the
      end); no behaviour is queued by this, so it does not count for the guard-spot effect above.
      A guard spot that is a cell holding an allied building is moved to the nearest free cell (not
      for vehicles whose type has the harvester flag; weeders are not excluded).

    **In our engine** row 5 never applies, row 3 applies only to a unit that holds the absorbing-building instruction (a neutral or special house unit, S-0605 claim 16), and no actor has the computer-owner effects (Terms,
    houses in our engine): the outcomes "row 5 applies" and the column "Computer-owner effects" of
    the table above are the original's only. **Original only (note for AI designers):** the
    original's computer player gets from them a saboteur that goes for a building it has as target,
    a guard spot moved off a cell holding an allied building, and a resting unit that follows its queued
    destinations back to its guard spot; our AI can get the same through sabotage, move and
    area-guard orders.

    When no row ends the processing, an actor with a guard spot is handled by the table below. The
    **leash** holds when the actor is not firing, has no destination, and its 3-D distance to the
    guard spot (with the building reduction of F4 when the guard spot is a building) is greater
    than the leash *L* (Formulas F5). The **search** is the area search around the guard spot of the
    target-acquisition specs, which also restarts the targeting timer (claim 33). It runs only
    when those specs allow the actor to search passively (as for *guard*, claim 33) and its
    targeting timer has run out; the column "may search" below means both.

    | Leash | Target | May search | Result in this processing |
    |---|---|---|---|
    | holds | any | yes | drops any target, takes the guard spot as its destination, and searches |
    | holds | any | no | drops any target, takes the guard spot as its destination, and attempts its idle action |
    | does not hold | none | yes | searches |
    | does not hold | none | no | attempts its idle action |
    | does not hold | present | any | takes the approach step of claim 37 and engages its target |

    A search that finds a target ends the processing with a delay of 1 tick; a search that finds
    nothing is followed by an attempt at the idle action. So an actor that the leash pulls back can
    take a new target around its guard spot in the same processing, and never engages, in that
    processing, the target the leash made it drop.

    An actor without a guard spot does none of this. A guard spot taken as the actor's own cell
    in this processing already qualifies for the table in that same processing (Q-0231).
    Thus, with no target or destination, an expired targeting timer and passive acquisition
    allowed, the actor searches around that cell and restarts the targeting timer with the
    search's draw (claim 33). An eligible enemy selected by that search becomes its target on
    this first processing, not a later one, and the delay is 1 tick with no delay draw. If the
    search finds nothing, the actor attempts its idle action on this processing, including
    an infantry unit's idle action as S-0107 defines it, then takes the ordinary delay specified below.
    If searching is not allowed, the no-target row still attempts the idle action. These
    results do not bypass the earlier rows or the infantry deployer rules.
    In a processing that none of rows 1 to 5 ends,
    any actor, with or without a guard spot, that has no target once the leash and the search have
    taken effect runs S-0518 claim 14's automatic charging scan when it is infantry or a vehicle
    with an electric-charge secondary weapon. A found own-house building becomes its target,
    its charging mark is set and *attack* is queued without an immediate start. The scan does
    not undo an idle action already attempted earlier in this processing. It leaves the
    area-guard delay below in effect, including its close-range test on the resulting target.

    The delay is *D* = *k* × *D*<sub>r</sub> + *U*(1, 5), where *k* = 2 for aircraft and 1 for
    every other kind; it becomes trunc(*D* / 6) under the same close-range condition as in *attack*
    (Formulas F3).
    - **Aircraft** (S-0501 claims 18 and 45, and its table T2): the rows, the tables and the
      delay of this claim, with *k* = 2, reach an aircraft's *area guard* only at a height other
      than its cruise height, above or below it, and only when no held target makes it queue
      *attack* (with a delay of 1 tick). At exactly its cruise height, each processing of an
      aircraft's *area guard* is instead the fall-back to rest, with a delay of 1 tick. At any
      other height, an armed aircraft with its ammunition spent and no contact makes the same
      fall-back first, in the same processing.
    - **Infantry** of a type that deploys: the deployer rules of claim 31 take precedence over every
      row. When one of them applies, it decides the outcome and the delay, and nothing else of this
      claim happens.
    - **Computer-controlled owners** (original only) also have the computer-owner effects of the
      table above, in the processings that the table marks, and at no other processing.
    - Backing: `measurable` (scenario B10, with case h for the close-range shortening, case i for
      the aircraft factor, case j for the bunker row and case k for the 1-tick delay after a search
      that finds a target; row 3 is the rule of claim 24, scenario
      B21; scenario B24, case b and S-0518 claim 14's TC2 and TC4 for automatic charging,
      and scenario B26, cases b and c for the
      original-only row 5 and the guard spot moved off an allied building; scenario B31 for row 1
      and the "wakes its slaves anyway" effect, with the wake-up rule of S-0402 claim 14 and its
      scenario H7). The computer-owner
      effect on queued destinations is `unmeasured`: no setup is known that has the guard spot as
      a computer-controlled unit's first queued destination. That the 1-tick delay after a search
      that finds a target takes no draw is `measurable` by scenario B27, case f: the search of a
      player house's actor takes only its timer draw and none for the objects it tests or scores
      (S-0309, claim 12, and its State and ordering), so the stream on that tick tells a draw of
      the delay apart from the search's. The newly taken own-cell spot's same-processing
      search, targeting-timer restart, acquisition and idle-action outcomes are `measurable`
      (scenario B34). Of the **Aircraft** bullet, the height test is `measurable` by S-0501
      scenario AC29 part j, as S-0501 claim 45 backs it. The rest of that bullet is
      `unmeasured`, as S-0501 claim 45 rates a computer owner's *area guard* in flight apart from
      its height test, and claim 18 rates T2's computer rows: the fall-back to rest at exactly the
      cruise height with its delay of 1 tick, the same fall-back made first by an armed aircraft
      with its ammunition spent and no contact at any other height, and the *attack* queued with
      a delay of 1 tick for a held target. No setup there can require a computer-controlled
      owner to perform a specified rest transition.
    - Confidence: high (the guard spot and delay rules and the newly taken spot's
      same-processing qualification, except for the **Aircraft** bullet); medium (the leash, the
      order of leash and search, the slave-master rows, the computer-only rules, the harvester
      rule and the aircraft's height test, as S-0501 claim 45 rates it); low (the rest of the
      **Aircraft** bullet, as S-0501 claim 18 rates the in-flight *guard* row's *area guard* for
      a computer-controlled house)

27. **Hunt.** At each processing, unless the actor's type has the stupid-hunt flag, it runs the
    normal search of the target-acquisition specs from its own position. The search restarts the
    targeting timer every time (claim 33) but only looks for a new target when the actor has none,
    so an actor that already has a target keeps it. The outcome then depends on whether the actor
    has a target after the search; an actor whose type has the stupid-hunt flag is treated as
    having none, even if it has one:
    - with a target: an infantry engineer (without explosive charges) or vehicle thief takes the
      target as destination and queues *capture*; an infantry unit with explosive charges whose
      target is a building takes it as destination and queues *sabotage*; in both cases with an
      immediate start if ready, which the processing's own delay then overrides (claim 3c). Any
      other infantry unit clears its destination and takes the approach step of claim 37; other
      kinds take that step,
    - without a target (or with the stupid-hunt flag): the actor only attempts its idle
      action. In a skirmish a hunting actor that finds nothing stays put, for every house.
      Campaign hunt outcomes are outside this spec's scope.

    The delay of this general rule is always the standard delay, with the rate of the behaviour
    that is current once the outcome above has taken effect: the rate of *capture* or
    *sabotage* when the outcome started one (claim 6).
    - **Vehicles** whose type deploys into a construction yard (or any deploying type that has a
      target or a human-controlled owner) instead try to find a clear spot and deploy there, and
      choose the standard delay. In our engine, where the owner of every player house counts as
      human-controlled, this covers every deploying vehicle of a player house. Deploying makes no other behaviour of the vehicle
      current: while it seeks the spot and rotates to deploy, *hunt* stays current, so the standard
      delay always takes the rate of *hunt*; when the deploy succeeds, the vehicle is replaced by a
      building of its deploys-into type (deploy and structures specs) and has no further
      processing. When the building may not be placed at the spot the vehicle has reached, the
      deploy is refused and the vehicle queues *guard* without an immediate start, so *guard*
      becomes current at its next per-tick start at which it is ready (claim 11). **Original only:** in addition, on every tick a
      vehicle whose type deploys into a construction-yard type queues *hunt* (without an immediate start) if its owner is
      computer-controlled, the game is not a campaign, the owner's automatic base building is on,
      the owner has no construction yard, and its current behaviour is neither *hunt* nor *unload*.
      A *hunt* queued this way becomes current through the same tick's per-tick start (claim 11) if
      the vehicle is ready to change.
    - **Infantry** of a computer-controlled owner (original only) have two rules of their own, judged on the target
      the unit has when the processing begins, which take precedence over the general rule above.
      When one applies (the first that does), it decides the outcome, the processing chooses 1 tick
      with no draw, and none of the general rule happens (no search, no targeting-timer restart, no
      idle action):
      - its target is a building and its type infiltrates, or occupies or assaults buildings and may
        occupy this one: it takes the target as destination and forces *capture* (the second row
        of claim 23's infantry table),
      - it is deployed (current sequence one of the deploy kinds) and has no target: it starts its
        *undeploy* sequence.

      The explosive-charges rule of claim 23 (the first row of its infantry table) is not among
      these. In *hunt* an infantry unit with explosive charges takes the general with-target
      outcome above instead, which queues *sabotage* with an immediate start if ready and chooses
      the standard delay.

    **In our engine** the per-tick *hunt* of base-building vehicles and the two
    infantry rules never apply to player houses (Terms, houses in our engine).
    **Original only (note for AI designers):** the original's computer player puts a
    base-building vehicle whose house has no construction yard (at the start of a skirmish,
    or after losing its last one) into *hunt* (so that it looks for a place to deploy), sends its building
    takers straight for a building they have as target, and packs up its deployed infantry when
    it hunts without a target. Our AI can get these results through move, deploy, capture and
    undeploy orders.
    - Backing: `measurable` (scenario B11; cases b and c for the infantry rules, case d for the
      vehicle deploy rule, case e for the general with-target outcome, case f for the original's
      per-tick *hunt* of base-building vehicles). The refused deploy remains `unmeasured`:
      the vehicle first looks for a spot where the building fits, and no setup is known
      in which the spot it reaches then refuses the building (Open questions).
    - Confidence: high (the general rule and the skirmish case); medium (the special cases, and
      that deploying makes no other behaviour current); low (the refused deploy)

28. **Capture and sabotage.** Here a *building taker* is an infantry unit with explosive charges, an
    engineer, an occupier, an assaulter or an infiltrator. At each processing, the row is chosen by
    the destination and target the actor has when the processing begins, and only that row applies.
    In the second row, whether the hunt rule also applies depends on the target the actor is left
    with after its fall-back to rest:

    | Destination | Target and actor | Outcome |
    |---|---|---|
    | none | the target is a building and the actor is a building taker | the target becomes its destination |
    | none | any other case | it falls back to rest; if its position is then on a cell that holds a building, it is sent the scatter request below; if it then has no target (the fall-back to rest can clear it, claim 15 rows 4 and 7), the hunt rule applies as well |
    | present | a target | no change |
    | present | no target | the hunt rule applies |

    **Scatter off a building.** The request of the second row has no threat point, is forced and is
    not move-anywhere. The cell tested is the one that holds the actor's position after its
    fall-back to rest. An infantry unit judges the request by S-0208 claim 11 and, when it accepts,
    finds its destination by S-0208 claim 13 (the case without a threat point). For a vehicle the
    scatter property does not decide, since the request is forced (claim 19); its other acceptance
    conditions and its destination are S-0207 claim 16 (the nearby-cell search, without a draw).
    An aircraft, which is never a building taker, judges the request by S-0501 claim 49. The
    request gives it no destination and no take-off of its own. Its effect, a second fall-back
    to rest when its current behaviour after the first one has the scatter property, makes only
    the choice of S-0501 table T2 again. It leaves the state the first fall-back gave it, except
    for a landed, armed aircraft with its ammunition spent whose search found a pad: when that
    second fall-back happens, that aircraft ends with the same pad, *enter* and the pad's
    contact place held (S-0501 claim 49). The hunt rule then follows for a computer-controlled
    owner as below, except that an aircraft whose *retreat* from S-0501 table T2 became current
    at the fall-back ignores the *hunt* request (S-0501 claims 34 and 49).

    **Hunt rule (original only):** an actor of a computer-controlled owner, other than one of an
    occupier infantry type, has its destination cleared and queues *hunt*. Where it has fallen back
    to rest or scattered in the same processing, this clears any destination that gave it, and
    *hunt* replaces whatever that queued (claim 7). An actor of a human-controlled owner, or of an
    occupier type, is left as the other effects leave it. **In our engine** the hunt rule never
    applies, so every actor is left as the other effects leave it. **Note for AI designers:** the
    original's computer player puts an actor in *capture* or *sabotage* that has lost its building
    into *hunt*; our AI can do so with an order.

    The delay is always the standard delay. Building-arrival effects are separate from this
    processing: S-0509 claims 1–11 own general entry and engineer outcomes, and S-0510
    claims 1–10 own infiltration and other ordinary entrants. S-0508 claims 1–5 and 14–17
    own garrison entry and assault; S-0509 claim 17 owns vehicle-like building destinations.
    S-0519 claim 13 owns charge planting at walking-step arrival, including its non-shot
    rearm replacement and delay draws without burst advance; these are not sabotage-processing
    effects or part of this processing delay. *Eaten* is processed by the same table,
    except that occupiers, assaulters and infiltrators do not count as building takers and the hunt
    rule never applies. Its second row sends the same scatter request.
    - Backing: `measurable` (scenario B12; scenario B29, case c, for *eaten* and for the
      fall-back of a vehicle in *capture*). An aircraft's part of this claim has S-0501 claim
      49's backing. The first fall-back's outcome on the ground with no pad contact, for an
      unarmed aircraft, is `measurable` (S-0501 scenario AC31 part a). The rest is `unmeasured`,
      as S-0501 claim 49 rates it: whether the request is sent and the aircraft's answer to it
      (no setup is known whose record differs between a sent and an unsent request), the second
      fall-back's end state with the pad, *enter* and its contact place held, the armed aircraft
      with its ammunition spent, the pad-contact and in-flight cases, *hunt* queued without an
      immediate start, and the computer-controlled case in which T2 made *retreat* current,
      with its *hunt* request ignored.
    - Confidence: medium (the table, and an aircraft's part of this claim), except the
      outcome where the first fall-back of an aircraft in flight takes S-0501 table T2's in-flight
      *guard* row's *area guard* for a computer-controlled house: low, as S-0501 claim 49 rates
      it; high (the options of the scatter request)

29. **Enter.** An infantry unit or vehicle in *enter* repeats its entry request at each
    processing, to its contact when present, otherwise to the object it remembers entering.
    Only acceptance or an existing tether permits continued entry: without a tether, any
    non-accepting reply ends contact and makes it fall back to rest. This includes a transport
    reply that makes the transport approach the requester. The transport protocol and its
    timing are S-0507 claims 4–6; aircraft entry is S-0501 claim 24.

    **Repair buildings, land and naval.** Permission is S-0314 claim 16, with S-0504 claim 7's
    naval-match test. The repeated request above is not a repeat of that initial permission
    query: merely standing on the depot does not make it fail. S-0314 claim 17 owns
    repair-specific tether establishment; contact alone does not establish a tether, and
    tethering need not wait for stopping or settling. Contact, arrival and servicing are distinct:

    | State | Event | Result |
    |---|---|---|
    | permission given, not yet in contact | successful entry request | contact takes the first free place before physical arrival; the depot has max(d, 1) places for imported docking-place count d (S-0314 claim 17) |
    | a one-place depot already holds another vehicle's contact | second vehicle makes an ordinary click | permission fails; no second enter order, waiting queue or displacement of the first contact (S-0314 claims 16–17) |
    | approaching with contact | destination guidance | S-0314 claim 18 supplies the building destination and per-vehicle docking point for ordinary land depots and naval yards, and the separate obsolete-unloading exception |
    | moving into the supplied docking-point cell | movement-arrival exchange | building queues repair and sets its ready mark; vehicle queues sleep, neither with an immediate start; no map-east body turn is requested, even when tethered; exact horizontal alignment with the docking point and a pre-existing tether are not required (S-0314 claims 17–18) |
    | completing entry | separate completed-entry confirmation, when supplied | confirmation can establish tether and turn the vehicle toward 16384 facing units (map east), with arrival reported once no longer moving, already tethered to the contacted building and still in enter; an ordinary repair depot without resource-unloading or obsolete-unloading flags supplies no such confirmation through repeated entry requests, with or without the pad flag or an existing tether (S-0314 claim 17) |

    Ordinary movement arrival schedules no later map-east turn merely because it queued
    *sleep*. If ready at that update's late start (claim 11), the vehicle starts *sleep*
    there without another *enter* processing. An already-turning driving vehicle can instead
    hold *sleep* queued under claim 11's ordinary readiness rule until the first permitted
    start after its turn ends. Arrival creates neither that turn nor another readiness gate.
    Ship and hovering locomotion do not inherit the driving-only body-turn hold; their
    other readiness conditions still apply (S-0314 claim 17).

    **Tank bunker.** The movement-arrival test applies also when the contacted building is a
    tank bunker: it is made when the vehicle's movement brings it into a cell, not by the
    *enter* processing. The "vehicle queues sleep" of the table above does not apply there: the
    bunker sets its ready mark and queues *repair* without an immediate start, and the vehicle
    stays in *enter* and queues nothing (the bunker's rules, including its refusal while selling,
    are S-0606 claim 5). As at a depot, arrival does not clear the order-level destination (see
    goal-cell equality below). Its backing and confidence are those of S-0606 claim 5.

    Behaviour starts still obey claims 3, 7 and 11: the completion is not a forced start of
    either queued behaviour. The movement-arrival exchange establishes no tether. At an ordinary
    depot without the pad or unloading flags, a vehicle may therefore remain untethered through
    waiting, settling and working; none of those repair phases creates a tether (S-0314 claim 17).
    Neither the click nor the contact reservation is a paid repair.
    S-0314 claim 6 and S-0504 claim 8 own waiting, settling, stopping during repair and payment.
    Foundation passability is unchanged (S-0209 claim 5); a docking point does not teleport
    the vehicle or make every intervening foundation cell traversable.

    **Goal-cell equality (Q-0797).** In an accepted ordinary entry with contact and no
    further approach destinations, this processing does not remove the building destination
    just because the vehicle stands in the supplied docking-point cell, whether it entered
    that cell this tick or was already there. There is no due-processing cleanup tick for
    cell equality. The repeated accepted request can retain that destination through later
    processings. A movement-arrival exchange may queue *sleep* in the arrival tick, as
    above, but queueing it is not removal of the order-level destination. S-0207 claim 26
    distinguishes the cell-goal locomotion cleanup from this building goal, including its
    separate exact-point guard exception. The same claim owns destinations naming a
    vehicle, infantry unit or aircraft, including a transport: no-track equality with the
    supplied-point cell alone clears neither destination, regardless of the named unit's
    motion. Its exact-point exception concerns the mover resting in *guard*, not the named
    unit's state, and compares the mover with the retained locomotor point, not the named
    unit's current position. Refusal, replacement orders, entry guidance, transport boarding
    and servicing can still change destinations by their own rules; this is not a promise
    of permanent retention or a replacement for those rules.

    **Earlier guard transition.** An unarmed repair building in the waiting stage of its guard
    processing can queue repair before the completed entry report: a contacted mobile actor
    must still be in enter, stand still, and have truncated spatial distance strictly below
    64 leptons from the foundation centre (S-0314 claim 17). Distance uses floating-point
    Euclidean calculation in three dimensions, truncated toward zero to integer leptons,
    not the table square root. This processing chooses 1 tick without a delay draw. That
    distance is not the later waiting distance to the docking point and makes no paid step.

    **Failure and interruption.** The non-accepting-reply rule above also applies at repair
    buildings. A vehicle retaining contact does not acquire a new reservation timeout or
    automatic yard reselection from this processing. Path failures retain S-0212 claim 23's
    categories and claim 11a's queue/start distinction; failed movement is not by itself an
    unconditional force to guard. For replacement move, attack and enter orders, S-0213
    claim 5 owns the ordinary order changes, with depot details in S-0314 claims 20 to 23:
    - repeated enter to the same contacted depot is ignored apart from route termination only
      while the vehicle is already in *enter*, not while docked in *sleep*,
    - an untethered vehicle releases contact under any other replacement order, even while
      sleeping, settling or working at a no-pad depot. A move order cannot retain remote repair
      in that case. A tethered, docked vehicle normally retains contact and tether under new
      orders, with the replacement-entry and same-depot-entry exceptions of S-0314 claim 20,
    - a vehicle with a queued depot entry retries once per eligible vehicle turn, after
      that turn's movement, self-healing, due behaviour processing and completed locomotor
      restoration (S-0314 claim 23). This is separate from the current *enter* processing's
      repeated request above, not an orders-phase batch or another movement pass. Acceptance
      queues *enter* with a readiness-dependent immediate start and assigns the destination
      that turn, but does not process the new behaviour synchronously. A place freed before
      the retry is available in that tick; competing retries follow object-turn order, not
      order-arrival time. Still-active chrono phase-in skips retry and retains queued entry.
      inability to move from a parasite hold, pulse disable, deactivation, magnetic lift or
      locomotor replacement alone does not skip retry on an otherwise ordinary living turn.
      Erasure can exclude that turn before retry is reached. Movement-caused open-topped
      passenger position updates precede retry; an attached parasite's same-turn delivery
      follows it, so a hold or lethal hit from that delivery cannot undo contact already taken.
      This adds no passenger-sync phase or second movement. S-0314 claim 23 owns these
      exclusions and ordering consequences, and inactive-depot clearing,
    - S-0314 claim 22 owns the run-away notice on sale: clearing a depot destination,
      changing *sleep* or *enter* to *guard*, and the possible scatter request. It also owns
      release on destruction and existing-contact power consequences. S-0314 claim 21 owns
      movement and servicing while contact remains.

    Stop is S-0213 claim 8, not a special depot-cancellation command. S-0314 claim 23 owns
    simultaneous-order contention and queued entry; claim 17 owns contact capacity and its
    multi-place servicing qualifications.
    S-0314 claim 24 finds no automatic repair return for a player-house vehicle: the name
    *retreat* supplies no such trigger, and computer-player scripts remain out of scope.

    Every outcome of this enter processing chooses the standard delay, using the behaviour
    current when the delay is chosen (claim 6).
    - Backing: `measurable` (scenario B12; S-0507 T5 and T6 for loading replies,
      S-0314 H16 and H19 for reservation, tether and movement arrival, H18 for orders,
      queued entry, run-away notices and other interruption consequences, H22 for late
      queued retries, phase-in gating, passenger/parasite ordering and next-turn processing,
      and scenarios B39 and B43 for depot processing, orders and goal-cell retention, plus
      S-0207 D31 for unit-goal retention and the exact-point guard exception, inheriting
      claim 26's measurement limitations).
      For transport loading, the existing-tether exception inherits S-0507 claim 5's
      `unmeasured` backing; repair-specific tether observations are the H19 cases above.
      Repair-specific tether, movement arrival and docking-point selection inherit
      S-0314 claims 17–18's confidence and measurement limitations. The interruption
      details inherit S-0314 claims 20 to 23's confidence and limitations, including
      the unresolved condition for sending the run-away scatter request.
      Multiple-place servicing retains S-0314 claim 17's qualifications, not an extrapolation
      from the sequential one-place case.
    - Confidence: high (reply rule, completed arrival and queue/start distinction),
      medium (reservation timing, early guard transition and ordinary approach)

30. **Unload (infantry).** For an infantry type that deploys, *unload* switches between deployed and
    undeployed, and in every case forces *guard* and clears the unit's destination:
    - a deployed unit (current sequence one of the deploy kinds) starts its *undeploy* sequence only
      if its type's undeploy delay is negative,
    - an undeployed unit starts its *deploy* sequence. If its deploy weapon (Terms: its secondary
      weapon, or its primary weapon for a type with the spray-attack flag) has the area-fire flag,
      a unit of one type singled out by name (Import mapping; `quirks.named_deploy_area_fire`)
      aims at nothing new, and a unit of any other type takes its own cell as its target.

    The delay is the first that applies in this table:

    | Unit | The type's undeploy delay | Area-fire flag of the deploy weapon | Delay |
    |---|---|---|---|
    | deployed | any | any | 450 ticks |
    | undeployed | 0 or more | any | the undeploy delay, in ticks |
    | undeployed | negative | yes, and the type is the one singled out by name | frame count of its *deploy* sequence + 1 |
    | undeployed | negative | yes, any other type | 450 ticks |
    | undeployed | negative | no (or no deploy weapon) | 450 ticks |

    Because the processing forces *guard* (claim 3c), the first processing of *guard* comes when that
    delay runs out. Any other infantry unit does nothing in *unload* and chooses 450 ticks with no
    draw, as the default processing of claim 4 does.
    What *unload* does for vehicles (transports unloading, deploying vehicles) and buildings belongs
    to the transport, deploy and structures specs (claim 34).
    - Backing: `measurable` (scenario B12; case b for the rows of an undeploy delay of 0 or more
      and of a deploy weapon without the area-fire flag, case c for the type singled out by name,
      case f for the slot that gives the deploy weapon and for the 450 ticks of an area-fire deploy
      weapon of any other type)
    - Confidence: high (the delay table and the forced *guard*); medium (the area-fire target rule
      and the choice of the deploy weapon)

31. **Deploying infantry in guard and area guard.** For an infantry unit of a type that deploys (the
    deployer flag), these rules take precedence over the rest of its *guard* or *area guard*
    processing. When one applies (the first that does), it decides the outcome and the delay, and
    nothing else of claim 24 or 26 happens; when none applies, the processing is as claim 24 or 26
    says.

    | Unit | Condition | Result | Delay |
    |---|---|---|---|
    | deployed | its type's undeploy delay is 0 or more | starts its *undeploy* sequence | frame count of that sequence |
    | deployed | undeploy delay negative, its type deploys to fire and is not immune to radiation | runs its deployed-firing rule (S-0517 claim 16) | standard delay |
    | deployed | undeploy delay negative, deploys to fire, immune to radiation, and its cell has radiation of level at least *R* | nothing | *D*<sub>r</sub> + *U*(10, 20) |
    | deployed | undeploy delay negative, deploys to fire, immune to radiation, its cell has no radiation or a level below *R*, and the firing rules let it fire at its own cell | fires its secondary weapon at its own cell during this processing, drops the target in this processing after the shot, starts its *deployed-fire* sequence | frame count of that sequence |
    | deployed | the same, but the firing rules do not let it fire | drops the target | *D*<sub>r</sub> + *U*(10, 20) |
    | undeployed | **original only:** computer-controlled owner, type deploys to fire, not immune to radiation, undeploy delay negative, no destination, no guard spot or standing on the guard spot's cell, and the ticks since the start tick exceed the auto-deploy delay *A* for the owner's difficulty index | if it is moving, it is told to stop; otherwise it starts its *deploy* sequence | standard delay if moving; else frame count of the *deploy* sequence |

    **Own-cell shot timing (Q-0983).** The successful radiation-immune row makes the shot
    on the tick of the processing that decides to fire, not on a later sequence fire frame.
    No fire-frame field applies to this shot: neither a frame of the *deployed-fire* kind
    nor a standing, prone or secondary-weapon fire frame delays it. The secondary weapon
    is used even for a type with the spray-attack flag. The own cell is the target for
    the firing check and this immediate shot only. The processing clears that target
    after firing and before starting *deployed-fire*; it does not retain it until an
    animation frame. The sequence is presentation of the shot already made, not a
    second pending shot under S-0107 claim 11. Its integer frame count is used unchanged
    as ticks until the next processing, not as a shot-release delay (Terms).
    The refusal row likewise clears the temporarily assigned own-cell target in the
    deciding processing. A later independently acquired target follows its usual rules.

    Radiation creation or top-up follows the shot's detonation under S-0525 claims 1
    and 4. For an own-cell shot that detonates immediately, the field therefore exists
    or is topped up on this processing's tick; it does not wait for *deployed-fire* to
    reach any fire frame. This does not replace projectile travel or detonation rules
    for other imported weapons.

    In the radiation rows, “its cell has radiation of level at least *R*” means that the unit's
    cell is the centre cell of a field whose current integer field level is at least *R*
    (S-0525 claim 7). Exposure from fields centred elsewhere and residue left by ended fields
    do not count. “No radiation or a level below *R*” means no such centre field, or its current
    field level below *R*.

    Here *R* = trunc(*W* / 3), integer division truncating toward zero, with *W* the radiation level
    field of the unit's secondary weapon (Terms; always the second slot, whatever the spray-attack
    flag says, so for a spray-attack type it is not the deploy weapon of claim 30), and *A* is the entry of the game-wide auto-deploy delay
    list for the owner's difficulty index (Terms; Import mapping), in ticks. If the list has no entry
    for the owner's difficulty index (it is empty or too short), *A* is unspecified in the original
    (Terms). In our engine only the units of the neutral and special houses can reach the row
    (below), and both houses have index 1, so our engine reads only the list's second entry. "The ticks since the start
    tick exceed *A*" means *t* > *t*<sub>s</sub> + *A*, with *t*<sub>s</sub> the start tick and the
    sum *t*<sub>s</sub> + *A* formed in the domain of F0 (a signed 32-bit sum that wraps). A deployed unit whose type
    has a negative undeploy delay and does not deploy to fire, and an undeployed unit of an immune
    type, follow claim 24 or 26 unchanged.

    **In our engine** the last row applies to no player house (Terms, houses in our engine), so
    this claim never deploys an undeployed unit of a player house, AI houses included. The units of
    the neutral and special houses keep the row, as they keep the original's other rules for owners
    that are not human-controlled, with *A* the list's entry for their difficulty index, which is
    1 for both (Terms, *difficulty index*; S-0412, claim 5). When the list has no second entry,
    the original's *A* is unspecified, and our engine gives the safe, deterministic result that
    the glossary's *unspecified* asks for: the importer warns and the row does not apply, so those
    houses' infantry never deploys by this row (the same choice S-0412, claim 3, makes for its
    short percentage list). This is a documented departure with no switch, proposed for the
    project lead's confirmation (Questions for the project lead). **Original only (note for AI
    designers):** the original's computer infantry of such a type deploys by itself once it has
    stood on its guard spot, in *guard* or *area guard*, for longer than the auto-deploy delay of its
    house's difficulty index since that behaviour started; our AI can get the same with a deploy order.
    - Backing: `measurable` (scenario B12, case b, for the first row and its delay; scenario B19
      for the other rows, with case d for the radiation-immune unit that the firing rules do not
      let fire; case e for immediate shot timing, independence from fire frames,
      target removal and same-tick radiation from immediate detonation).
      `unmeasured`: the original's *A* when the list has no entry for the owner's difficulty index,
      because an unspecified value cannot be confirmed by any result (as in claim 36, part b). Our
      engine's result for that case is a design rule and needs no backing.
    - Confidence: high (the first row and the auto-deploy row's delays, the immediate
      secondary shot, absence of a fire-frame wait and same-processing target removal),
      medium (the radiation rows
      and the auto-deploy conditions, and that a missing entry gives no defined *A* in the original)

32. **Buildings in guard and attack.** A building processes *guard*, *sticky* and *area guard* by the
    same rule (table T1), with the properties of whichever is current.
    - **Armed, in *guard*:** every processing sets the building's ready mark (claim 11). If the
      building has a target, it queues *attack* and starts it at once, whether or not it is ready
      to change (claim 7), and chooses 1 tick; else it chooses the anti-air delay
      *D*<sub>aa</sub> + *U*(0, 2), where *D*<sub>aa</sub> is F1 applied to *a*, the anti-air rate of
      the current behaviour (claim 21). A building whose type is an electromagnetic-pulse cannon,
      some superweapon buildings (structures specs) and a building that can be occupied but has no
      occupants choose the anti-air delay whether or not they have a target, and so do not start
      *attack* by this rule.
    - **Unarmed, in *guard*:** a building whose type has the stupid-guard-mode flag chooses 100 ticks
      and does nothing else. Any other unarmed building runs the factory, repair-bay and reload
      rules of the structures specs (which can queue *repair* and end the processing with 1 tick),
      including S-0417 claim 13's idle apron clearing for a vehicle factory in *guard*, *sticky*
      and *area guard*, and chooses the standard delay if its type is a unit repair building, else
      3 × *D*<sub>r</sub> + *U*(0, 2). Neither sets the ready mark.
    - **In *attack*, a building whose type does not have the SAM flag:** without a target it queues
      *guard* and starts it at once, whether or not it is ready to change, and chooses 1 tick. With
      a target, every processing sets the ready mark and asks the firing check (Terms) whether the
      building can fire at the target now. The answer's category decides the outcome; no delay
      takes a rate or a draw:

      | Firing check's answer (Terms) | Result | Delay |
      |---|---|---|
      | *can fire* | fires subject to the upgrade weapon gate below (combat specs own the shots) | 1 tick |
      | *not facing* or *rearming* | rotates towards the target | 2 ticks |
      | *illegal*, *impossible*, *out of ammunition* or *out of range* | drops its target, queues *guard* and starts it at once, whether or not it is ready to change | 1 tick |
      | any other answer (*busy*, *cloaked*, *moving*, *must deploy*, *still turning*) | rotates towards the target where that applies | 1 tick |

      So an attacking building is processed every tick or every other tick, and one whose target
      leaves its weapon's range returns to *guard* at its next processing at which the firing check
      answers *out of range*. While its rearm countdown runs, that check answers *rearming*
      instead, unless one of the building's rank-4 reasons of S-0301 claim 9 gives its own answer
      (ranks 1 to 3 apply only to vehicles; S-0301 claims 1 and 9), such as a target test's
      *illegal* or *impossible*. While its animation-delayed shot is pending, the check answers
      *rearming* unless one of S-0405's table T1 answers holds (S-0301 claims 9 and 14). The check
      can also answer *rearming* for the other rank-5 causes of S-0301 claim 9, such as the
      weapon's own effect still being alive. While the check answers *rearming*, for any cause, the
      building keeps its target.

      For a building that S-0309 claim 29 covers, that claim's periodic held-target check can
      drop a target that the firing check refuses before this processing does. A holder in any
      behaviour of this spec other than *capture* and *sabotage* makes that check, whether or
      not it aims a weapon at the target (S-0309 claim 29). No rule of this spec changes for it.

      **Upgrade weapon gate.** For a non-SAM building the gate applies only when its
      upgrade count is nonzero and upgrade position 1 is occupied (S-0406 claim 12).
      A zero count or empty position 1 bypasses it, even if higher positions hold upgrades.
      It never substitutes the first occupied higher position or the most recently installed
      upgrade. The *can fire* row tests position 1's ordinary primary and secondary
      weapons, not its elite replacements, to decide whether firing is allowed. The
      gate passes only when that upgrade has a primary weapon and at least one of these
      conditions holds: both slots name the same weapon type, the primary's imported
      burst count is greater than 1 shot, or a secondary exists whose imported burst
      count is greater than 1 shot. These are integer comparisons, with no rounding.
      This tests position 1 even if another eligible position supplies the weapon
      chosen for the target (S-0308 claim 12). Sales, skipped positions, replacement and
      an excess wall-tower count follow S-0406 claim 12; they do not change the gate's
      position-1-only test or supply any fourth valid upgrade position.

      When the gate passes, this processing makes a primary firing attempt followed by a
      secondary firing attempt against the target, without another firing check between
      them. Both use the building's weapons as resolved by S-0308 claim 12. A rearm
      countdown begun by the primary does not prevent the secondary attempt. When the
      gate fails, neither attempt occurs: the building retains its target and *attack*,
      still sets the ready mark and chooses 1 tick. It can therefore remain in *attack*
      indefinitely with a *can fire* answer but no shots. When the gate does not apply,
      the ordinary selected-weapon firing rule remains in force. This rule is not specific
      to map placement.

      **Animation-delay priority (Q-0798).** A passing gate makes the pair immediately
      in that processing even when the building has the animation-delayed-fire flag.
      It starts no delayed shot: neither a delayed pair nor a replacement delayed single
      shot. A failed gate with the original quirk setting starts neither attempt nor
      delayed shot. A bypassed gate uses the ordinary path, including animation delay
      when enabled (S-0301 claim 14). If the resolved secondary slot is empty, its attempt
      does nothing: no shot, burst advance, rearm change, ammunition use, random draw or
      firing-check answer (S-0301 claim 9). A missing secondary on the gate-tested upgrade
      alone is insufficient for this no-op: another eligible provider or the host may
      supply the resolved slot (S-0308 claim 12).

      The suppression is `quirks.upgrade_attack_weapon_gate`, default original (on).
      With it off, a failed gate instead permits the ordinary selected-weapon attempt,
      including the ordinary animation delay when enabled (S-0301 claim 14),
      while a passing gate still permits the immediate paired attempts without starting
      a delayed shot. This changes gameplay, not only
      presentation.

      **Turret snap** (low confidence). When the answer is *not facing* and the building is a
      turreted building whose own type has the rules voxel-turret flag (Terms; shared with
      S-0310 claim 4 and S-0313 claim 10), the building checks its aim error *ε*, the
      angle between its turret facing and the direction to the target in facing units, taken the
      shorter way round (0 to 32 768). If its type's turn-rate field is 0, or *ε* is within the
      snap bound *b* of Formulas F8, its turret facing is set exactly onto the direction of the
      target at once (no gradual rotation over ticks), and the processing has the outcome of the row of the
      table above that the firing check's answer for the new facing selects, so the building
      can fire in this processing (first row, 1 tick) instead of rotating (second row, 2 ticks).
      Otherwise the first answer stands. No other answer, and no building without a voxel turret,
      takes this snap. *Rearming*, which shares the second row, never snaps.
    - **In *attack*, a building whose type has the SAM flag:** this processing never sets the ready
      mark. It keeps one of two stages in the progress count (Terms), *aiming* (0) and *aimed* (1).
      Every behaviour start puts it in *aiming*. A processing in either stage has the result of
      the first matching row for its stage and chooses 1 tick, with no rate and no draw. "A
      suitable target" means a target that is an aircraft whose world height is not 0.

      | Stage | Condition | Result | Stage afterwards |
      |---|---|---|---|
      | *aiming* | the type needs power, it drains power (its type's power value is negative, Import mapping), and its owner is short of power (Terms) | nothing: the processing starts no rotation and drops no target, even one that is no longer suitable | *aiming* |
      | *aiming* | no suitable target | drops its target, queues *guard* and starts it at once, whether or not it is ready to change | (*guard* is current) |
      | *aiming* | a suitable target | rotates towards it | *aimed* once it has stopped rotating and its facing is within 8192 facing units of the direction to the target; else *aiming* |
      | *aimed* | no longer a suitable target | drops the target | *aiming*, where the next processing queues *guard* |
      | *aimed* | the firing check answers *can fire* | fires both its weapons, primary and secondary, in this processing (see below; the combat specs own the shots themselves) | *aiming* |
      | *aimed* | the firing check answers *not facing* | nothing | *aiming* |
      | *aimed* | the firing check answers *illegal*, *impossible* or *out of range* | drops the target | *aiming* |
      | *aimed* | any other answer (*rearming* included) | nothing | *aimed* |

      In the *aimed* stage the rows of the table above take the firing check's answer for the
      primary weapon only. On *can fire* the building makes two shots in that processing, one with
      each weapon; which of the two comes first is not fixed here (the combat specs own the shots
      and any per-shot rule for which the order matters). The secondary
      weapon's own state never holds back its shot: its shot is made although the primary shot has
      just started the rearm countdown (decided by the project lead; recorded under Questions for the
      project lead).

      **Out-of-stage rule.** When, at a processing, the progress count holds neither stage (a
      value other than 0 and 1), the building does nothing in that processing, leaves the count
      as it is, and chooses the anti-air delay *D*<sub>aa</sub> + *U*(0, 2) (claim 21) instead of
      1 tick, which takes one draw. A start resets the count and the rows above set only 0 and 1,
      so this happens only when *attack* became current by an override (which keeps the count,
      claim 10) from a behaviour whose own rule had left another value; since the count stays as
      it is, every later processing of that *attack* does the same until a behaviour start
      resets the count. Among the behaviours this spec states, only a gate's *open* leaves such a
      value (its stages *waiting* to *finished*, claim 35); the *repair* and *missile* processings
      of buildings also leave some (structures and superweapons specs). Whether any stock situation
      reaches the rule is open (Open questions).

      The combat specs own the firing itself.
    - Backing: `measurable` (B36 and its extensions for the upgrade gate's predicate,
      suppression, position-1 and ordinary-weapon selection, ordered paired attempts,
      readiness and delay, and installation during play; S-0406 PL16 for the position/count
      cases; S-0301 F17 for empty-secondary effects and animation-delay priority). The remaining
      rules are `measurable` (B36's unarmed follow-up for the absence of a ready-mark
      setter inside unarmed guard; scenarios B8 and B16: cases b and f for the unarmed delays, case g for
      an armed *guard* building that starts *attack* by its own processing, case a for the rows
      *can fire* and *rearming*, case e for the out-of-range row and the target kept during the rearm countdown, case h for the turret snap and
      the row *not facing*, case i for *still turning* in the row of other answers; for SAM
      *attack*, case c for the 1-tick intervals and the return to *guard*, case j for the power
      row, case k for the aiming tolerance, case l for firing both weapons and case m for the
      out-of-stage rule, with a test gate type that has the SAM flag; no other setup here reaches
      that rule). These parts are `unmeasured`:
      - secondary refusals other than rearming and range: B16 l and B45 d make the unchecked
        secondary shot `measurable` despite rearming and despite its own range, respectively.
        No fixture isolates every other secondary refusal category,
      - the out-of-stage rule after a *repair* or *missile* processing: the structures and
        superweapons specs own those processings and the values they leave, so no setup here can
        put a building in such a stage on purpose,
      - the buildings that choose the anti-air delay even with a target (the electromagnetic-pulse
        cannon, the superweapon buildings and the empty occupiable building), because the only
        observable difference from the general rule needs such a building to hold a target at a
        processing, and no setup is known that gives one,
      - a pending animation-delayed shot's *rearming* against a target out of range: no scenario
        here drives an attacking building's target out of range while its animation-delayed shot
        is pending; it rests on S-0301 claim 9, which rates that shot's place against the later
        ranks medium; S-0301 scenario F21 tests the shot kept and dropped against the refusal
        rows of S-0306 claim 23, not against *out of range*,
      - the non-SAM answers *illegal* and *impossible*: no fixture supplies a separately
        identified target producing either answer. *Out of ammunition* is `measurable`
        (B45 e), separately from B16 e's *out of range* case,
      - the SAM answer-specific transitions other than *can fire*, rearming and target loss:
        B16 does not isolate *not facing*, *illegal*, *impossible*, *out of range* or each
        remaining answer in the *aimed* stage. Its timing runs cannot distinguish these
        individual reasons without recording and setting up the corresponding firing-check
        result at that stage,
      - in the non-SAM row of other answers, the answers *busy*, *cloaked*, *moving* and *must
        deploy*. S-0301 (claim 9) gives *moving* only to a firer that moves and *must deploy* only
        to a vehicle, so no building gets them; it gives *busy* to a firer launching spawned
        aircraft, and no building is known to launch any; and when a building is cloaked, and
        whether it uncloaks to fire, is owned by S-0516, and no scenario here isolates a
        building's *cloaked* answer at a known processing. The row keeps
        the table total.
      The periodic held-target check of S-0309 claim 29 takes that claim's backing: `measurable`
      for *area guard* and *move* (S-0309 scenario A6 part p, run p8), `unmeasured` for the
      other behaviours, as that claim gives.
    - Confidence: high (upgrade gate applicability, predicate, suppression, paired-attempt
      order, empty-secondary no-op and animation-delay priority),
      medium (armed and unarmed *guard*, the ready mark, non-SAM *attack* and its delay
      table, a pending animation-delayed shot's *rearming* against a target out of range,
      which is medium because no scenario here makes that case, SAM *attack* every tick and its
      stages, firing both weapons, and the periodic held-target check of S-0309 claim 29 as
      that claim rates it); low (the turret snap and
      its bound F8, the SAM aiming tolerance, the power condition and the out-of-stage rule)

33. **Automatic search under behaviours.** Separately from processing, on every tick an actor
    whose targeting timer has run out acts by one of two rules of the target-acquisition specs:
    - an actor that carries an attack-move memory (S-0213, claim 6) acts on that memory as S-0309
      (claims 1 and 23) gives, whatever its current behaviour, which runs a search only in some
      cases. S-0309 owns that rule: when its search runs, what it does to the actor's target and
      behaviours, and that the passive-search conditions below do not limit it,
    - any other actor whose current behaviour is *move*, *harvest* or *guard* runs the opportunity
      search, if the target-acquisition specs allow it to search passively (S-0309, claims 2 and 3:
      nearly always for *guard*; for *move* and *harvest* only under the conditions given there,
      such as the opportunity-fire flag).

    *Sticky* is not in this list, although it is processed as *guard*. *Area guard* and *hunt*
    search from their processing instead (claims 26 and 27). Every search covered by S-0309
    claim 4 restarts the targeting timer with *N* + *U*(0, 2) ticks, where *N* is the game-wide
    area-guard targeting delay if the current behaviour is *area guard* and the normal targeting
    delay otherwise (Import mapping). The charging break-off search of S-0518 claim 15 and
    S-0309 claim 1 runs inside infantry attack processing without waiting for or restarting the
    targeting timer, and takes no targeting-timer draw. Its replacement can be attacked in that
    same processing. Claim 33's observable ordering of processing draws before per-tick-search
    draws does not govern this inside-processing search. Which targets a search leaves as opportunity targets is owned by
    S-0309 (claims 23 and 24). On each tick, an actor whose target is an opportunity target drops it if its current
    behaviour is *ambush*, *construction*, *enter*, *harmless*, *missile*, *open*, *repair*,
    *selling*, *sleep*, *stop*, *unload* or *wait* (an actor whose current behaviour is *none*
    keeps it).

    **Order within the actor's update.** For one infantry unit, vehicle or building on one tick,
    the per-tick start (claim 11), the drop rule above, the processing (claim 2) and the per-tick
    search relate as follows; each statement is stated by its observable effect:
    - (a) the drop rule judges the behaviour that is current once the per-tick start has taken
      effect (for a building, its first per-tick start of the tick), and that tick's processing
      never acts on a target the drop rule removed on that tick,
    - (b) whether the per-tick search runs is decided by the current behaviour as that tick's
      processing leaves it (before any late start of claim 11, which therefore never influences
      the search or the drop rule of its own tick), so a processing that starts *guard* (for example a building's *attack*
      whose target left its range, claim 32) can be followed by a search, and a restart of the
      targeting timer, on the same tick,
    - (c) a target that the per-tick search finds on a tick is first acted on by the processing of
      a later tick, never by the processing of the same tick, and the drop rule of the same tick
      does not remove it,
    - (d) when both the actor's processing and its per-tick search draw on the same tick,
      the processing's choices receive the earlier stream values, and the per-tick search's
      choices receive the subsequent stream values. This includes the search's *U*(0, 2) for
      the targeting timer. The choices and resulting random-stream state must match these
      assignments; no internal execution order is prescribed.

    For an actor that carries an attack-move memory, parts (a), (c) and (d) hold as stated, with
    its attack-move search as the per-tick search: a behaviour that this search starts with an
    immediate start is a start after that tick's processing, so it is first processed on a later
    tick (claim 3, part b). Part (b) does not apply to it, since whether its search runs does not
    depend on its behaviour.
    - Backing: `measurable` (scenario B10 for the timer reset and the behaviour lists, with case e
      for the drop rule and part a; scenario B27, case a for part d, case b for part b and case c
      for part c; the attack-move exception by S-0309, scenario A6, parts d, g and l; the
      paragraph on actors with an attack-move memory by scenario B27, case e; the charging
      break-off exception by S-0518 scenario TC3 and S-0309 scenario A17)
    - Confidence: high (the timer reset, the charging break-off exception, the behaviour lists, the attack-move exception and parts a
      to d); medium (the passive-search conditions, owned elsewhere, and the paragraph on actors
      with an attack-move memory)

33a. **Dropping allied targets for nonhuman owners.** **Original only (note for AI designers):**
    on each update that reaches ordinary target maintenance, an actor of a computer-controlled
    owner drops its current target when that owner counts the target as allied, including
    its own-house actors. This does not wait for the behaviour-processing or targeting timer.
    A target removed here cannot be used by that update's subsequent behaviour processing.
    A later search may still choose a new target on the same update.

    The following are exceptions: this rule does not clear the target when
    - the actor is berserk (S-0610 claim 6),
    - the actor is an engineer infantry unit,
    - the actor is infantry and the target is a building that currently permits that infantry
      unit to occupy it (S-0508 claim 1), or
    - the actor has a secondary weapon with an electric-charge warhead and the target is an
      overpowerable building (S-0518 claims 11 and 12).

    For a passenger in an open-topped transport that is itself mind-controlled, the controlling
    actor's house replaces the passenger's own house for both the human-control gate and the
    alliance test. Otherwise the passenger's own house is used. Alliance is tested from the
    house just selected toward the target's owner, not the reverse. A cell target has no
    allied owner and is not removed by this rule.

    In our engine this nonhuman-owner gate holds only for the neutral and special houses.
    AI player houses follow the human-controlled rule and do not clear targets on this ground
    (Terms, houses in our engine). The exceptions do not promise that another target-loss rule
    cannot remove the same target.
    - Backing: `measurable` for the berserk exception (S-0610 scenario P4).
      The ordinary clearing rule, the other exceptions and the passenger-house substitution
      are `unmeasured`: no validated skirmish fixture here isolates an already-held
      allied target at this maintenance point without another target-loss rule intervening.
      Scenario B41 gives a conditional investigation, not a claim of a completed fixture.
    - Confidence: high (clearing, exception conditions and house selection); medium (observable
      timing relative to behaviour processing)

### Behaviours owned by other topics

34. **Pointers.** The processing of these behaviours is part of other topics and is not specified
    here. Their properties (claims 16 to 21) and the slot rules (claims 7 to 11) still apply.
    - *harvest*: economy, the harvest cycle,
    - *construction*, *selling*, *repair*: structures (build-up, selling, repair bays and service
      depots); *open* for buildings is claim 35,
    - *unload* for vehicles: S-0507 claims 7–12 for transports, and S-0517 claims 3 to 8
      for in-place deploying; aircraft passenger unloading: S-0507 claim 13; for factory buildings:
      S-0417 (structures, factory unloading); other building unloading: structures.
      A vehicle transport's *finished* processing queues *guard* with the allowance to
      interrupt movement, but still chooses the standard delay of claim 6, including its
      scenario draw (S-0507 claim 11). Aircraft passenger unloading also uses that standard
      delay except for the *stopped check* and *reselect* phases of S-0507 claim 13:
      each chooses exactly 1 tick with no delay draw,
    - *missile*: buildings only (superweapons and structures); infantry, vehicles and aircraft
      process it by the default rule of claim 4 (table T1),
    - *paradrop approach*, *paradrop overfly*, *spy-plane approach*, *spy-plane overfly*: aircraft
      only (S-0501 claim 34 for spy-plane flight; superweapons and paradrops
      own the remaining effects); the other kinds process them by the default rule of claim 4,
    - *retreat* for aircraft: S-0501 claim 34; other *retreat*, *rescue* and *patrol*: team logic
      of the original's AI (out of scope) and the
      route orders of the order specs. *Patrol* for vehicles would restart an unfinished unload
      or deploy cycle without changing its ordinary outcome or delay, but that cycle is always
      finished for vehicles, so the restart has no effect (S-0517 claim 15). The deployed or
      transitioning vehicle's *move* fallback does not extend to *patrol* (S-0517 claim 10),
    - aircraft-specific *attack* and run completion: S-0501 claims 12–18; *enter* and pad
      docking: S-0501 claim 24; *guard*, *hunt*, *move*, *retreat* and spy-plane flight:
      S-0501 claims 29–34. In particular, aircraft *hunt*'s choice queues *attack* without an
      immediate start; it becomes current at a permitted opportunity of S-0501 claim 31,
      not by bypassing readiness. Aircraft fall-back and queued starts are S-0501 claims 18
      and 31 (claims 11 and 13 here); aircraft passenger *unload*: S-0507 claim 13.
    - Backing: pointers inherit the cited claims' backing; the transport completion delay
      and aircraft phase exceptions are `measurable` (S-0507 scenarios T10 and T11,
      recording processing ticks and scenario random state).
      The aircraft *hunt* queue/start
      distinction is `measurable` (S-0501 AC10, recording queued and current behaviours against
      the readiness and start opportunities of S-0501 claim 31).
      The default processing it names for the other kinds is the rule of claim 4 (table T1). The
      inactive vehicle cycle is `measurable` (S-0517 P6); a restart cannot be timed because no
      vehicle rule starts that cycle.
    - Confidence: high (that these behaviours have their own processing, and that aircraft *hunt*
      queues without an immediate start); medium (the inactive vehicle cycle)

### Gates

35. **Gate state and the *open* behaviour.** A gate (S-0209 Terms) has a *door* that is, at any
    moment, at rest closed, at rest open, opening or closing. Its *passage state* is **open** when
    its current behaviour is *open* and its door is at rest open; in every other case it is
    **closed**. The passage state is what the gate rows of S-0209 (claims 4 and 5) call an open or
    a closed gate. So a gate whose door is still moving, in either direction, is closed for S-0209,
    and so is a gate whose door stands open while another behaviour is current.
    - (a) **Start.** Every gate starts with its door at rest closed, whether the map places it or a
      house builds it. Only the *open* processing below moves the door.
    - (b) **Who opens it.** The *closed friendly gate* outcome of S-0209 makes the gate open; no
      other rule is known to start *open* on a gate. Map building entries do not carry a
      behaviour (claim 4); the general data vocabulary in Import mapping does not add such
      a field. When that outcome arises and the
      gate's current behaviour is not *open*, or its door is closing or at rest closed, the gate
      drops any queued behaviour, and *open* becomes current at once, whether or not the gate is
      ready to change (a behaviour start, so the progress count is 0 and the gate is processed at
      the next opportunity, claim 3). When *open* is already current and the door is opening or
      at rest open, nothing changes.
    - (c) **Processing of *open*.** A building that is not a gate queues *guard* and chooses a delay of
      1 tick. A gate is processed in stages; the stage is its progress count, and every
      behaviour start puts it in stage *start*. *T*<sub>d</sub> is the door time and
      *T*<sub>c</sub> the close delay of F9, and "Std" is the standard delay of claim 6, with the
      rate of *open*.

      | Stage | Situation at the processing | Effect | Next stage | Delay |
      |---|---|---|---|---|
      | start | door at rest open | close countdown restarted | waiting | Std |
      | start | door at rest closed | door starts opening (it will take *T*<sub>d</sub> ticks); close countdown restarted | opening | 0 |
      | start | door opening | close countdown restarted | opening | 0 |
      | start | door closing | door reverses to opening (see the quirk below); close countdown restarted | opening | 0 |
      | opening | door at rest open | none | waiting | 0 |
      | opening | door at rest closed | queues *guard* | finished | 0 |
      | opening | door still moving | none | opening | 0 |
      | waiting | an object other than the gate is in the ground layer (S-0209 Terms, cell layer) of one of the gate's foundation cells | close countdown restarted | waiting | Std |
      | waiting | no such object, and the close countdown has run out | none | closing-start | Std |
      | waiting | no such object, countdown still running | none | waiting | Std |
      | closing-start | any | door starts closing (*T*<sub>d</sub> ticks) | closing | 0 |
      | closing | door at rest closed | queues *guard* | finished | 0 |
      | closing | otherwise | none | closing | 0 |

      The close countdown is restarted with length *T*<sub>c</sub> and has run out once at least
      *T*<sub>c</sub> ticks have passed since its last restart; with *T*<sub>c</sub> = 0 it has run
      out at once. It is looked at only in stage *waiting*: the first *waiting* processing that
      finds the countdown run out and the cells empty moves the gate to stage *closing-start*, and
      the door starts closing at the next processing, one Std later. The
      countdown starts when the door starts opening, not when it is fully open. A *finished* gate
      changes nothing at any further processing in *open*, and each such processing takes the
      delay Std; the queued *guard* becomes current by the slot rules (claims 7 to 11).
      Each Std takes one draw *U*(0, 2) from the scenario random stream (F2); the other delays (0,
      and 1 for a building that is not a gate) take none. A delay of 0 means processing again at
      the gate's next turn, as a delay of 1 does, except in a second turn of the same tick (claim
      2).
    - (d) **Door motion.** A door that starts opening or closing reaches rest open or rest closed
      at the gate's own turn (S-0203) on the first tick on which *T*<sub>d</sub> ticks have passed
      since the motion began (with *T*<sub>d</sub> = 0, at its first turn after the start). The
      door moves at every turn of the gate, whatever its current behaviour and whether or not the
      gate processes in that turn (claim 2). A processing of the gate in the turn in which its
      door reaches rest already finds the door at rest. So, for a door that starts opening in the
      processing of tick *t* with *T*<sub>d</sub> = 90, the stage-*opening* processing of tick *t* + 90 finds it at rest open
      and moves to stage *waiting* with a delay of 0, and the first *waiting* processing, with its
      Std and that Std's draw, comes in tick *t* + 91 (a door that reached rest only after the
      processing would put it in tick *t* + 92). The same holds for a closing door and stage
      *closing*. The per-tick starts of claim 11 neither look at the door nor move it, so where the
      door's motion falls relative to them has no observable effect. Where the gate's turn falls
      within the tick, and so whether another object's turn in the same tick sees the door still
      moving or already at rest, is S-0203's. The drawn stage is drawing only.
    - (d1) **Drawn door position.** Let *G* be the imported gate-stage count, a positive integer,
      and *e* the integer ticks since the current opening or closing began. For an uninterrupted
      motion of positive duration *T*<sub>d</sub>, define
      *q* = R(*min(e, T*<sub>d</sub>*) / T*<sub>d</sub>*) and
      *n* = trunc(R(*q × G*)). The division and multiplication are separate floating-point
      operations with a 53-bit significand, not integer arithmetic: R rounds each result
      toward zero with `quirks.chop_rounding` on, or to nearest, ties to even, with it off
      (S-0205 claim 4). The final conversion truncates toward zero to an integer.
      The unclamped position is *p = n* while opening and *p = G − n* while closing.
      The displayed position is max(0, min(*p*, *G* − 1)), as S-0108 claim 4 requires.
      It is evaluated for the draw's tick, not just at behaviour processings.

      Thus opening begins at 0 and closing begins at unclamped *G*, displayed as *G* − 1.
      At *e = T*<sub>d</sub>, the opening expression is *G* and the closing expression is 0,
      but the gate's turn on that tick has already brought the door to rest under part d.
      Before that tick the opening expression is less than *G*; it need not visit *G* − 1
      when there are fewer motion ticks than image stages. While *open* is current, a door
      at rest open continues to display position *G* − 1, with the same damaged-image selection
      as a moving door in S-0108 claim 4. That rule is not a moving-frame request of *G*.
      When *open* is not current, or the door is at rest closed, S-0108 claim 2 supplies
      the ordinary stationary body image.
    - (d2) **Position on reversal.** With `quirks.gate_reverse_snaps` on, the closing-to-opening
      reversal of part c immediately gives *q = 1*: its opening position is unclamped *G*,
      displayed as *G* − 1, even before the next gate turn brings it to rest open.
      There is no intermediate retracing image. With the switch off, if closing had run for
      *a* ticks before reversal, the reopening takes *a* ticks and uses the opening expression
      of d1 with progress *q* = R(min(*T*<sub>d</sub>*, T*<sub>d</sub>* − a + e*) / *T*<sub>d</sub>*),
      where *e* now counts ticks since reversal. This off-switch rule is a design alternative,
      not an additional original behaviour.
    - (d3) **Zero-time motion.** For *T*<sub>d</sub> = 0, use *q = 1* without division, including
      on the motion-start tick. Starting motion does not itself put the door at rest: its first
      later gate turn does (part d). A draw between those turns therefore does use a moving-door
      image, at position *G* − 1 for opening and 0 for closing. There is no sweep through
      intermediate images. A display need not present that tick, so the statement concerns
      the image selected when a draw occurs, not a guaranteed visible refresh.
    - (e) **Consequences.** A friendly unit at a closed gate waits about *T*<sub>d</sub> ticks
      (S-0209: it waits without a new path search) and then passes. A gate stays open as long as
      something stands in its foundation cells, and closes no earlier than *T*<sub>c</sub> ticks
      after it began to open. More precisely, it begins closing at the processing after the
      first *waiting* processing that finds its cells empty with the countdown run out, one Std
      later; reaching rest closed then takes the door time of part d. A unit that reaches a closing gate makes it reverse
      by (b).

    **Quirk `quirks.gate_reverse_snaps`** (default on, the original). A door that reverses from
    closing to opening reaches rest open at the gate's next update (part d), however far it had
    closed. Off: the reopening takes as many ticks as the closing had run until the reversal, so
    the door retraces its path. Drawing
    follows the door either way.
    - Backing: `measurable` (scenario B28: case a for parts a and b and the opening time, case b for the
      close delay and its counting from the opening start, case c for the open-while-occupied rule,
      case d for the reversal quirk, case g for where within the gate's turn the door reaches rest (part d), case f for the conversion of a gate time that does not fit,
      and scenario B30, case b, for a gate time whose product with 900 reaches 2⁶³).
      `unmeasured`: the non-gate row of (c), since nothing established here
      gives a building that is not a gate the behaviour *open*, map building entries carry no
      behaviour, and no other input that reaches this row is established; and the "door at rest closed" row of stage *opening*, which no
      known situation reaches (only a reversal stops an opening door, and only stage *start*
      reverses)
    - Confidence: high for part a, the passage state and the door time; medium for part b (stated as its
      net effect on the slots), the stage table, the countdown rules and the
      quirk; high for where within the gate's own turn the door reaches rest (part d, Q-0030),
      and for the drawing formulas, reversal position, rest-open image and zero-time image
      (parts d1–d3, Q-0685; `measurable`, scenario B28 h–j). The off-switch retracing
      formula is a design rule.

### Game-wide settings with engine defaults

36. **Area-guard threshold and guard stray distance when no layer gives them.**
    - (a) The area-guard threshold of claims 14 and 15 has the engine default 4. So, when no rules
      layer gives it, an armed vehicle whose owner's house skill level is 3 or lower rests in
      *guard* by row 10 of claim 15 (unless it has the guard-area ability or a type that defaults
      to area guard), and one whose owner's level is 4 or higher rests in *area guard* by row 12.
    - (b) The guard stray distance of F5 has **no engine default in the original**: when no rules
      layer gives the key, the value the original uses is unspecified (Terms): it depends on
      unrelated internal state and cannot be reproduced. **In
      our engine** (decided under the project lead's delegated sign-off, 2026-10-10), when no layer
      gives the key, the leash *L* of an actor whose guard spot is a mobile actor is the area-guard
      leash of F5 as for any other guard spot, *L* = ⌊11 × *G* / 10⌋ leptons, and the importer
      warns. This is a documented departure (Terms), with no switch, since the original's value is
      unspecified.
    - Backing: (a) `measurable` (scenario B28, case e). (b) `unmeasured`: the original's value in
      this case is not defined, so no measurement can confirm a value; our engine's replacement is
      a design rule and needs no backing
    - Confidence: high

### Approaching the target

37. **The approach step (Q-0072, questions 1, 4 and 5, and the behaviour side of question 3,
    Q-2113 question 1, behaviour side, and Q-2171 question 4).**
    - (a) **When it is taken.** An infantry unit or vehicle takes the approach step at every
      processing of *attack* in which it has a target after the charge-target check (claim 23,
      row *approach*), so at the cadence of that processing's delay (claim 23 and F3), not on
      every tick. The *area guard* and *hunt* processings take it too where claims 26 and 27 say
      the actor approaches its target (the *guard* processing does not). Separately, on every tick, an infantry unit or
      vehicle that has a target and no destination, is not a team member, and either cannot reach
      its target's cell (part a1) or is a passenger of an open-topped transport, takes the
      approach step in every behaviour; if it still has no destination afterwards and its target
      is out of range (part b), it drops the target. Aircraft, and buildings, never take the
      approach step (part g).

      (a1) **Cannot reach (Q-2171 question 4).** The actor cannot reach its target's cell when
      S-0212 claim 6's comparison, made with the actor in place of the unit and the target's cell
      in place of the destination cell, finds the two in different reach regions of the actor's
      movement zone. So the actor's cell is read for its deck (S-0212 claim 4) when the actor is
      on a bridge deck, and the target's cell whenever that cell carries a high bridge, whichever
      layer the target is on. Two cells of no reach region count as the same region, and so do
      two off-map cells (S-0212 claim 4). The comparison is skipped, and the target counts as
      reachable, in claim 6's three exceptions. First, the actor's type has no movement zone,
      which never happens in our engine (S-0212 departure D1). Second, the actor's cell is
      outside the usable area but a legal cell. Third, the actor is in the corridor-disabling
      state of S-0212 claim 14 (a train, a loaner, or current behaviour *retreat*: its team
      condition does not arise, since a team member is excluded above), its cell is inside the
      usable area, and the target's cell is a legal cell outside it. The actor's cell is the
      cell of one point. While the actor travels through a tunnel (S-0212 Terms), it is the
      tunnel exit's cell centre. Otherwise it is the point the actor's locomotor is moving it
      towards at that moment, while there is one, and otherwise the actor's own position. For a
      walking infantry unit that point is its walking target (S-0208 Terms), not the end of its
      walk, so for infantry the point is its scatter reference point (S-0208 Terms). For a
      vehicle it is its heading point as its movement spec gives it (S-0207 Terms for driving,
      S-0503 Terms for hovering). So it is the actor's own cell while it has no movement in
      progress.

      So an actor on a deck reaches a target on the same bridge when both cells are read through
      enabled links whose first ends lie in one reach region (as S-0307 claim 19 has it for its
      candidates), and an actor on a bank cell of the region that the target's deck read gives
      reaches a target on the deck or on the water under it. Two cells of one bridge need not
      read through the same link, so being on the same bridge alone does not settle it.
      Apart from the three exceptions, an actor on a cell of no reach region reaches a target on
      another such cell, and no other.
      A target on the water under a high bridge is read for the deck as well, so a vessel on that
      water counts as unable to reach it unless the deck read gives the vessel's region. These
      results follow S-0212 claim 6, which has no switch, and have none here.

      (a2) **Place in the tick (Q-0161).** The per-tick rule above judges the target,
      destination and current behaviour left after the per-tick start, any processing on that
      tick, and the per-tick search of claim 33. It follows that search, not precedes it.
      On a tick without processing its place is unchanged: after the per-tick start and
      the per-tick search opportunity. If no search runs, it still judges the state then held.
      A processing's approach does not exempt its resulting state from this rule.
      If the processing left a destination, or no target, this later rule does not apply.
      Otherwise its destination or target-loss outcome must still hold on that tick.
      This specifies the resulting state, not a count of internal evaluations. Thus a target found by the per-tick search can receive an approach
      destination, or be dropped, on that very tick, before any later behaviour processing
      acts on it. The final out-of-range drop applies only if no destination remains after
      this step; finding a target does not protect it from this rule.
    - (b) **In range.** "In range" in this claim is the range test of S-0307 (claims 1 to 12) from
      the actor's current position, with the source point the firing check uses (S-0307 claim 8)
      and with the weapon chosen for the target (S-0308). It is not the whole firing check: an
      actor whose weapon is rearming, which is still turning to face the target, or to which the
      firing check gives any answer other than one that comes from that range test, is in range
      for this claim if the range test passes. A blocked line of fire or a bridge counts only as
      far as S-0307's range test refuses it.
    - (c) **Nothing to do.** The step does nothing when the actor has no target, or when the actor
      grips a unit with a magnetic beam (it is that unit's lifter, S-0607) and its target is that
      gripped unit. So a lifter never backs away from, or closes on, the unit it holds because of
      the range test, minimum range included, and the approach step never gives it a destination
      that would end the grip (S-0607 claim 12). Part h is decided before this, so a lifter type
      with one of part h's approaches could still take its gripped unit as destination (medium,
      `unmeasured`: no scenario here gives a lifter a part h approach). When the actor is a seized
      unit and its target is its own lifter, the step runs as for any target. A destination it gives
      has no effect, since a seized unit ignores destinations (S-0607 claim 9). The step's drop
      (part d) and its failed search with no nearby cell (S-0307 claim 17) give a gripped unit no
      destination, so while the unit has a destination they end its grip (S-0607 claim 9).
    - (d) **Whether the actor keeps its target.** A target in range is never dropped by the
      approach step. A target out of range is kept or dropped as the table says. Here an actor
      *may approach* when its type has the can-approach property (Import mapping) and it is not
      draining another object (for example a power-draining vehicle linked to a building), not
      sitting in a bunker and not a passenger of an open-topped transport. All three exclusions
      apply whatever the property says. The table reads the actor's current behaviour, whichever
      processing or per-tick rule took the step.

      | Current behaviour | May approach | Owner | Target out of range |
      |---|---|---|---|
      | *sticky* | either | any | dropped, with the destination |
      | *attack* | either | any | kept |
      | *area guard* | yes | any | kept |
      | *area guard* | no | human-controlled | kept |
      | *area guard* | no | computer-controlled | dropped, with the destination |
      | *hunt* | yes | any | kept |
      | *hunt* | no | computer-controlled | kept (**original only**, below) |
      | *hunt* | no | human-controlled | dropped, with the destination |
      | any other | yes | any | kept |
      | any other | no | any | dropped, with the destination |

      **Original only:** in our engine no player house is computer-controlled, so an actor of a
      player house in *hunt* that may not approach always drops its target here; the neutral and
      special houses keep the original's rule (Terms). Dropping clears the target and the
      destination at once, and nothing else of the step happens; what the actor does next is its
      behaviour's processing (for example the fall-back of claim 23 at its next *attack*
      processing). An actor that keeps its target goes on to parts e and f in the same step.
    - (e) **An existing destination is kept unless the target has moved too far from it.** For an
      actor that holds a destination and kept its target:

      | Can-recalculate property | Target | 3-D distance from target to destination | Destination |
      |---|---|---|---|
      | yes | out of range | more than the renewal threshold of F10 | cleared, and part f applies in the same step |
      | yes | out of range | at most that threshold | kept |
      | yes | in range | any | kept |
      | no | either | any | kept |

      The can-recalculate property is described in words in the Import mapping. A kept destination
      ends the step for an actor that is not in the air (S-0307's airborne test): nothing else
      happens in this step, **also when the target is in range**. The approach step never clears a
      destination because the target came in range; movement outcomes are part i's pointers. An
      actor in the air that keeps its destination is treated by part f as if it had none. So a
      target that moves is followed by renewing the destination only at processings of the behaviour
      (part a), and only once it is more than the threshold away from the destination; the per-tick
      case of part a never renews a destination, because it applies only to an actor without one.
      Part e's clearing removes the destination directly, not by an assignment of an empty
      destination, so it is not an effective clear in S-0502 claim 24's sense and no jumpjet stop
      follows it (S-0607 claim 9).
    - (f) **No destination.** For an actor that kept its target and has no destination (none to
      start with, or cleared by part e), or that is in the air:

      | Situation | Outcome |
      |---|---|
      | the target is in range, the actor is in the playfield (Terms), and the actor is not the deploy-to-fire vehicle of part f2 | destination unchanged: an absent one stays absent, and an airborne actor's held one stays held |
      | not the case above, and the actor is a vehicle with at least one approach destination (Terms) | it takes the first approach destination as its destination, that entry leaves the list, and the step ends |
      | neither of the cases above | the actor searches for an approach position; the covered search results and failure outcomes are S-0307 claims 15 to 25, with only the deploying-search gap remaining; claim 17 supplies the failure centre after candidate trials as well as with empty radii |

      (f2) **Deploy-to-fire vehicle.** A vehicle whose type deploys to fire (S-0301), standing on
      a cell where its deploy test fails (S-0301, the must-deploy answer), searches as in the
      last row above even when the target is in range, and its *r*′ of F10 is replaced by
      ρ₃ + 512 leptons whenever ρ₃ + 512 ≤ *r*′, where ρ₃ is its 3-D distance to the target
      (F4). The same *r*′ also sets its renewal threshold in part e.
    - (g) **Actors that do not approach.** A building never takes the approach step: what a
      building does with a target out of range is claim 32. An aircraft's approach belongs to
      the aircraft specs. A deployed infantry unit (its current sequence one of the deploy kinds)
      of a human-controlled owner takes no approach step at all while deployed: it does not move
      towards a target out of range (claim 23, third infantry row, and S-0517 claim 16).
      **Original only:** a deployed infantry unit of a computer-controlled owner whose
      type fires when deployed undeploys when its target is out of range (our AI can order an
      undeploy). Zero type speed is not an exclusion from the approach step. An otherwise
      ordinary, non-deployed infantry unit or vehicle with zero type speed takes the same step
      and can receive a destination from the same search. Zero speed does not mean that its
      movement zone is absent or that its target is unreachable. The step alone supplies no
      movement: actual displacement depends on the locomotor and the speed rules (S-0207,
      S-0208). This is distinct from setting the can-approach property to no, which still permits
      approach in *attack* (part d). A building, by contrast, never receives an approach
      destination: a non-SAM building drops an out-of-range target and starts *guard* once its
      firing check answers *out of range*, and a SAM building follows its aim-then-check stages
      (claim 32).
    - (h) **Special vehicle approaches.** Before the parts above, a vehicle may take its target
      object itself as destination instead, and then nothing else of the step happens: the
      crushing approaches of S-0210 claim 11 (its attack-target variant for the original's
      computer-controlled houses, which is the original's only, and its omni-crusher variant), and
      a balloon-hover type whose weapon chosen for the target has a vertical projectile (S-0307
      claim 18), including a cell target, which becomes its destination without a search. Search failure can also give hunter-seeker types and vehicle-thief infantry the
      target object as destination (S-0307 claim 17), but that is not an earlier replacement of
      the step.
    - (i) **Coming into range during movement.** The step does not require arrival before
      firing, and coming into range does not itself remove a destination (part e). For driving,
      sailing and hovering vehicles, S-0207 claim 19 owns the subsequent movement. Driving
      and sailing stop at a track target, clear their destination and discard remaining steps
      when the target is mobile, the current behaviour (or queued one if none is current) is
      *attack*, *area guard*, *hunt* or *rescue*, no approach destinations remain, and that
      claim's arrival range test passes. They keep their target. Hovering discards stored steps
      at step points while in range, subject to that claim's chase exception. A transition
      leaving no next step applies the same conditional stop; when those conditions fail,
      a running retry delay can leave the destination kept while the vehicle waits. Choosing
      a next step in that transition does not apply the stop. Neither rule stops a vehicle
      between arrival points solely because it came into range. Firing during movement
      remains subject to the firing check (S-0301, S-0310). S-0208 claims 16
      and 17 and scenario W10 own infantry stopping: an ordinary walker in *attack*, *area guard*, *hunt* or
      *rescue* clears its destination and keeps its target on the first arrival tick at which
      a mobile target passes that claim's range test. Normally that test measures to the
      target's cell centre, not its own reference point; open-topped types have the separate
      test stated there. Non-mobile targets cause no early stop. This arrival rule does not
      change the approach step's retention rule of part e. S-0208 claim 21 separately owns
      the walking consequence of S-0307 claim 22(b)'s successful close-range own-cell
      shortcut. Its choice of a walking point is not a new exception to parts a or e here.
    - Backing: `measurable` (scenario B32: case a for the cadence and the in-range rule, case b
      for the kept destination and its renewal, case c for the can-approach property in
      *attack* and *area guard*, case c2 for *hunt* with a human-controlled owner, case d for
      the per-tick rule with an unreachable target, case e for deployed infantry, case f for zero-speed types,
      case g for part a2's order on processing and non-processing ticks and same-tick loss
      of a newly found target, case h for part a1; B16 case e for buildings).
      Part a1's comparison and its deck reads through an enabled link are `measurable` (B32 h).
      Part a1's deck reads through a link that is not enabled or with no link found are
      `unmeasured`, for S-0212 claim 4's reasons. Its comparison of two cells of no reach region,
      or of two off-map cells, is `unmeasured`: no scenario here stands an actor that holds a
      target on such a cell. Its three exceptions are `unmeasured`: S-0212 claim 6's scenarios
      back them for path computations only, and no scenario here puts an actor that holds a
      target outside the usable area or in the corridor-disabling state while isolating the
      per-tick rule. A moving actor's cell (the point its locomotor is moving it towards, or the
      tunnel exit) is `unmeasured`: no scenario here gives an actor that is still moving a
      target out of range with no destination (S-0307's Open questions leave the moving case
      unmeasured for its search too).
      Part a2's independent eligibility and same-tick destination or target outcomes are
      `measurable` (B32 g). An unchanged in-range state imposes no requirement to repeat
      an indistinguishable evaluation.
      Part i's movement outcomes have S-0207 claim 19's and S-0208 claims 16 and 17's backing and
      confidence. Part c's magnetic-beam case is `measurable` (S-0607 scenario L17), not yet run.
      Part c's drop and failed search that end a grip, and part e's direct removal with no effective
      clear and no jumpjet stop, have S-0607 claim 9's backing: `unmeasured`, since no S-0607
      scenario makes a gripped unit take the approach step, and no scenario here records whether
      a jumpjet actor of any other kind stops at part e's removal.
      These parts are `unmeasured`: the part h exception to that case (no scenario here gives a
      lifter a part h approach); the draining case of part d (the
      draining link belongs to the structures and combat topics, and no scenario establishes
      that link while testing approach); the *sticky* and *any other*
      rows of part d (an actor in those behaviours takes the step only by the per-tick case of
      part a, whose own drop of a target out of range for an actor without a destination gives
      the same observable result for every row that drops, and for the rows that keep the target
      the outcome is S-0307's search); the in-the-playfield exception
      (no setup here has an attacker outside the usable area in range of its target); part f2,
      because no B32 case puts a deploy-to-fire vehicle on a cell that fails its deploy test
      while isolating the changed radius and renewal threshold; the computer-controlled rows
      of parts d and g, because B32 tests human-controlled attackers only and supplies neither
      a computer-controlled non-approaching hunting unit nor a deployed computer-controlled
      unit at an identified approach step.
    - Confidence: high (parts a, b, c apart from its magnetic-beam case and its grip-ending drop and
      failed search, d, e apart from its direct removal, and the outcomes of part f); medium (part
      c's magnetic-beam case and its part h exception, as S-0607 claim 11 rates them, part c's
      grip-ending drop and failed search and part e's direct removal with no effective clear and no
      jumpjet stop, as S-0607 claim 9 rates them, the per-tick rule of part a, the in-the-playfield
      mark, part f2, part g's deployed infantry and zero-speed types). High for part a2's ordering
      and independent eligibility. Part a1: medium for the comparison and its deck reads through an
      enabled link. Low for its deck reads through a link that is not enabled or with no link found,
      and for two cells of no reach region or two off-map cells, as S-0212 claim 4 rates them. Low
      for the three exceptions: S-0212 claim 6 rates them medium for path computations, but none of
      that claim's backing concerns the per-tick rule and no scenario here tests them. Low for a
      moving actor's cell.

38. **House-selected centre after path give-up (Q-0238 and Q-0312).** This complements
    S-0212 claim 23a and its final nearby-search row in T5. It applies to the original's
    non-human-owned infantry and vehicles and, in our engine, only to the neutral and
    special houses, not player houses. Both non-player houses use the same rule.

    **House records.** A house begins with no recorded base centre or base-start cell,
    and a recorded base radius of 0 leptons. A start assignment supplies a base-start
    cell (S-0009); having map buildings is not required for a start assignment. A
    separately set base centre takes precedence over the base-start cell. Its setting
    and clearing by map actions are owned by the trigger topic, not inferred from the
    unit's position. With no such action the centre stays absent.

    Entry or removal of an owned building, and a building's owner change for both
    affected houses, refresh the base-start cell and radius from the house's surviving
    on-map buildings with positive health. This refresh clears the previous base-start
    cell and radius, even if the cell came from a start assignment. With no owned
    buildings the cleared state remains. Otherwise the weighted horizontal mean and
    radius follow F12. The mean's cell is replaced by a nearby-cell result: *foot*
    class, *normal* movement zone, no reach-region restriction, deck mode, no height,
    no-overlay or burrowing condition, no reference cell, and no bridge exclusion,
    using S-0212 claim 25. That result, possibly absent, is the new base-start cell.
    No draw is taken by this refresh. Damage or a price-factor change alone does not
    refresh these records: the choice uses the last recorded values.

    **Selected centre.** Prefer the recorded base centre, then the base-start cell.
    The displacement starts from that cell's reference point at its terrain height.
    When neither exists, it starts from world position (0, 0, 0) leptons, not the
    failing unit's position. F11 gives the displacement, snapping and usable-area
    correction, including this no-base case. Thus the no-base answer is a
    map-dependent cell obtained by correcting a point near the world origin, not
    automatically no cell, a start waypoint, or a cell near the failed unit.
    Snapping makes both horizontal world coordinates cell-centre coordinates, so
    the generated world point cannot be empty; no empty-point retry or fallback
    occurs in this infantry/vehicle use. The centre is not yet the destination: S-0212's final search
    applies the unit's ground reach region and can still yield no destination.

    **Random assignments.** Let *R* be the recorded radius clamped to 768 … 2048
    leptons, in exact integer arithmetic. Infantry and vehicles always use this
    central choice irrespective of their weapons: there is no outer-choice index
    draw. One *U*(0, *R*) supplies the displacement radius, followed by exactly one
    raw scenario-stream value for direction. Only its low eight bits affect the
    direction, but the whole value is consumed. Neither a missing base/start nor a
    correction at a map edge suppresses these assignments. Conversion, correction,
    fallback and the final nearby search take no draw. The number of ranged requests
    and raw direction draws is the same for both houses and all maps. The number of
    raw values consumed by the ranged request can differ through S-0202's rejection
    rule; the draw ranges can differ through the recorded radius.
    - Backing: `measurable` (B37, with S-0212 P26 for the final search and timing).
      Precedence and retention of a separately set base centre, and a start-assigned cell
      before any building refresh, are `unmeasured`: B37 supplies no repeatable ordinary
      setup or map action that sets either record for a neutral or special house. A price-factor
      change alone not refreshing the records is also `unmeasured`: no scenario here changes
      that factor during a game while isolating it from a building event. Damage-only
      retention is tested separately in B37. These gaps do not qualify the covered
      no-base fallback, building refresh or draw rules.
    - Confidence: high for selection and draw assignments; medium for record-refresh
      geometry and usable-area correction

### Reactions to being hit

39. **Crush response to a hit (Q-2005).** When a hit on a vehicle has been applied and has not
    killed it (among the results of S-0302 claim 12, this includes *unaffected* and
    *unchanged*), the vehicle makes the **crush response** (Terms) to the hit's attacker when all of
    these hold (a hit that reports *now dead* or *post-mortem*, S-0302 claim 12, gives no
    response):
    - the hit names an attacker (S-0302 claim 1),
    - the vehicle is not a team member (Terms) and is not tethered (glossary, *tether link*),
    - its owner is not allied with the attacker's owner. A house counts as allied with itself, so
      a unit of the vehicle's own house never gets the response,
    - its owner is not human-controlled,
    - the vehicle and the attacker meet the conditions of S-0210 claim 11: the vehicle is a
      crusher, the attacker is crushable by it and does not appear disguised to its house, the
      distance between their centres is at most the crush range, and the table there allows the
      owner.

    **Effect.** The vehicle is given the attacker object itself as its destination, not a cell. When
    the destination is accepted, the response queues *move* (claim 7), with no immediate start. The
    destination is given under the rules that govern every new destination of a vehicle: a vehicle
    that is deployed, deploying or undeploying receives none (S-0207 claim 20 (a)), under S-0517
    claim 9 and S-0607 claim 9, and a seized vehicle ignores it (S-0607 claim 9). Whether *move* is
    still queued when the destination is refused is open (Open questions). The response itself does
    not change the vehicle's target; S-0607 claim 12 applies to a lifter that takes the destination.
    Nothing else about the vehicle decides it: the response is made in every current behaviour,
    whether or not the vehicle holds a target or a destination, and whether or not it is berserk.
    Among the conditions above, being a team member or tethered is the only "busy" exception. The
    response is the only occasion of S-0210 claim 11's first list: no behaviour's processing and no
    search makes that choice, and an enemy that does not hit the vehicle is never chosen.

    **Order within the hit.** The response follows every other effect of the same hit on the
    vehicle (S-0302), so:
    - when the hit makes the vehicle berserk, the berserk entry first drops its target and queues
      *hunt* (S-0610 claim 4), and the response then queues *move* in its place (claim 7, last
      row). A berserk hit on a vehicle that is already berserk queues nothing of its own, and the
      response queues *move* in the same way,
    - in the original, a team member that the hit makes berserk leaves its team at the berserk
      entry, so the team condition does not hold it back,
    - an ordinary hit may first make the vehicle retaliate (claim 18): it then holds the attacker
      as its target, under an *attack* override (claim 10), and the response's queued *move*
      becomes current at the next start (claim 11). A berserk hit causes no retaliation (S-0302
      claim 10),
    - a vehicle that makes the response does not, for that hit, make the return to a refinery
      that the original gives a damaged resource-gathering vehicle of a computer-controlled owner
      (economy topic, not specified here),
    - when several hits on the vehicle are applied in one tick, each one that qualifies makes the
      response in the order in which they are applied (S-0302 claim 19).

    **While it drives.** Once *move* is current, the vehicle follows the attacker as S-0207 claims
    25 and 26 give for a destination that names a unit (claim 22), and crushes it under S-0210
    claims 3 and 4 when it reaches it. The response does not make it fire or stop firing: whether it
    fires on the way follows its target and the firing rules (S-0301, S-0309), and the opportunity
    search of *move* (claim 33) sets no destination: what it does to the target is S-0309's. A later
    qualifying hit replaces the destination with the newer attacker. While the destination is held,
    the original's attack-target variant of S-0210 claim 11 does not apply, because it needs a
    vehicle without a destination.

    **In our engine.** Under the glossary entry *AI house*, the owners of every player house,
    human-controlled or AI house, count as human-controlled here, so by default their vehicles
    never make the response, as in the original for a human-controlled owner. With
    `quirks.auto_crush_computer_only` off (S-0210, Edge cases), the condition "its owner is not
    human-controlled" is dropped for a player house, and S-0210 claim 11 gives the permission that
    applies instead. The vehicles of the neutral and special houses and of test-only
    computer-controlled player houses make the response by the original's rule. The team
    condition never holds back a vehicle in our engine, as no house there has teams (Terms).
    **Original only (note for AI designers):** the original's computer-controlled crushers chase
    and crush the crushable enemies that hit them from close by; our AI may reproduce this by
    giving a hit crusher an ordinary move order that names the attacker.
    - Backing: `measurable` (S-0210 scenario K7 cases 1 to 4, 8 and 13 to 17 for the occasion, the
      destination and the owner conditions, and case 18 for no response without a hit, in *guard*
      and in *hunt*; K13 part a for the disguise condition; K15 for a berserk vehicle, the replaced
      *hunt* and the unchanged target). The alliance condition is `unmeasured` until a route for
      S-0210 K15 c is named: K15 c decides it once one is, and no other case here isolates it. The
      tether and team conditions and the harvester ordering are `unmeasured`: no scenario here holds
      a crusher tethered or in a team while an enemy hits it, and the harvester's own reaction is
      not specified. The order with retaliation and the case of several qualifying hits in one tick
      are `unmeasured`: K7's responding crushers are unarmed, so none retaliates, and no scenario
      here lands two qualifying hits on a crusher in one tick. The refused destination of a
      deployed, deploying, undeploying or seized vehicle is `unmeasured`: no scenario here hits a
      crusher in one of those states. A hit with the *unaffected* result is `unmeasured`: no
      scenario here lands a hit that leaves the crusher unaffected. The *post-mortem* result is
      `unmeasured`: S-0302 claim 12 finds no effect of it in a skirmish. That the response is made
      whatever the current behaviour, and whether or not the vehicle already holds a target or a
      destination, is `unmeasured`: K7's responding crushers are unarmed and idle with no
      destination, K7 case 18 covers *guard* and *hunt* only for no hit, and K15's crusher starts
      idle, so no scenario here lands a qualifying hit on a computer-controlled crusher that is busy
      with another target, destination or behaviour, and none tells the rule from one limited to
      idle crushers.
    - Confidence: high for the occasion (a hit, and only a hit, other than one with the *unaffected*
      or *post-mortem* result; K7 cases 1 to 4 and 18), the owner (except as below), disguise, range
      and crusher conditions, the destination naming the attacker and the queued *move*. The
      attacker's being crushable is not part of that high rating: it keeps S-0210 claim 2's ratings
      (medium for "not falling"), as S-0210 claim 11 states. Medium for the response in every
      current behaviour and whatever target or destination the vehicle holds, for the tether and
      team conditions, the replacement of the queued *hunt*, the order with retaliation, the berserk
      team member, several qualifying hits in one tick, the refused destination, a hit with the
      *unaffected* or *post-mortem* result, the alliance condition, the harvester ordering, and the
      owner condition for the neutral and special houses of S-0210 K7 case 17, which rest on S-0412
      claim 5 (as S-0210 claim 11 rates them); low for the behaviour while driving (firing and
      searches)

## Formulas

Notation: RZ₅₃(*z*) is *z* rounded toward zero to a double, as in S-0003 (the original's
round-toward-zero mode, whose single switch `quirks.chop_rounding` belongs to S-0205, number
domains). trunc(*z*) is *z* truncated toward zero to an integer. *U*(*a*, *b*) is one
ranged draw (Terms).

**F0. Integer domain of delays.** Every delay value, and every sum, product and quotient formed from
one (*D*<sub>r</sub> + *U*(0, 2), *k* × *D*<sub>r</sub> + *U*(1, 5), 3 × *D*<sub>r</sub> + *U*(0, 2),
*D*<sub>r</sub> + *U*(10, 20), 1 + *U*(1, 10), the halving and the division by 6, the targeting
delay *N* + *U*(0, 2) of F6, and the start-tick test of claim 31), is a signed
32-bit integer. Sums and products wrap modulo 2³² into −2³¹ … 2³¹ − 1 (two's complement). Divisions
truncate toward zero, so trunc(−7 / 2) = −3. A delay that comes out at 1 or less means "next tick"
(claim 2).

**F1. Rate delay.** Let *r* be the current behaviour's rate in game minutes, as parsed by S-0003's
decimal form (claims 6 and 7): a single-precision value widened exactly to double, or, when the
value contains `%`, the double product of claim 7. Then

> *p* = RZ₅₃(*r* × 900)  (900 = ticks per game minute)
>
> *D*<sub>r</sub> = *w*(trunc(*p*)) if |*p*| < 2⁶³;  *D*<sub>r</sub> = 0 if |*p*| ≥ 2⁶³ or *p* is not a number

where *w*(*q*) is the integer congruent to *q* modulo 2³² in −2³¹ … 2³¹ − 1 (the wrap of F0).
Units: *r* in game minutes, *p* and *D*<sub>r</sub> in ticks. The product is one double-precision multiplication, rounded toward zero with
`quirks.chop_rounding` on (the original) and to nearest with it off (S-0205). For a rate without
a percent sign the product is exact, so both settings give the same *p* (S-0205, claim 9); they
differ only for a rate with a percent sign. trunc truncates toward zero. For |*r* × 900| below 2³¹ the wrap does nothing.
The 0 for |*p*| ≥ 2⁶³ covers every rate of 2⁶³ / 900 game minutes (about 1.02 × 10¹⁶) or more,
and an infinite rate (S-0003 claim 6 turns a value beyond the largest finite single into
infinity). For a rate without a percent sign it agrees with the wrap: such a single is a whole
multiple of 2³⁰, so its product with 900 is a whole multiple of 2³² and *w* of it would be 0 too.
For a rate with a percent sign the product need not be such a multiple, so there the 0 and the
wrap can differ. Either way the 0 differs from a conversion that saturates (which would give
2³¹ − 1). *p* is never "not a number" for imported data: S-0003's decimal form
gives a finite value or an infinity, never a not-a-number, and so does its product with 900.
That case is listed only to make the conversion total. Example: a rate written as 0.06 is held as the single 0.0599999986588955; times
900 that is 53.99999879…, so *D*<sub>r</sub> = 53. A rate written as 0.05 is held as
0.0500000007450581 and gives 45. The engine default rate is the double nearest to 0.016 (not a
single), which gives 14. The anti-air delay *D*<sub>aa</sub> of claim 32 is this same formula
applied to the anti-air rate.

**F2. Standard delay.** *D* = *D*<sub>r</sub> + *U*(0, 2), in the domain of F0. Unit: ticks.

**F3. Close-range shortening.** In *attack*: *D* = trunc((*D*<sub>r</sub> + *U*(0, 2)) / 2). In
*area guard*: *D* = trunc((*k* × *D*<sub>r</sub> + *U*(1, 5)) / 6) with *k* = 2 for aircraft, 1
otherwise. Both in the domain of F0; unit: ticks. The draw is taken whether or not the condition holds. The
condition is: the actor has a target; it is an infantry unit whose type has the close-range flag,
or its primary weapon's range is at most 512 leptons; and 282 ≤ ρ₂ ≤ 768, with ρ₂ the 2-D distance
of F4 between the actor's and the target's positions. (The original makes this comparison in
double precision, and on the integer ρ₂ it selects exactly this band.) Because ρ₂ comes from the table square
root, a true distance of exactly 282 leptons gives ρ₂ = 281 and is not shortened, while 769 gives
768 and is.

**F4. Distances.** For positions (*x*₁, *y*₁, *z*₁) and (*x*₂, *y*₂, *z*₂) in leptons let

> *n*₂ = (*x*₁ − *x*₂)² + (*y*₁ − *y*₂)²,  *n*₃ = *n*₂ + (*z*₁ − *z*₂)²  (exact integers; the original sums them in double precision, which is exact below 2⁵³)
>
> ρ₂ = trunc(*r*(*n*₂)),  ρ₃ = trunc(*r*(*n*₃))

where *r* is the table square root: *r*(0) = 0, and for *n* ≥ 1, *r*(*n*) is *n* truncated toward
zero to 14 significant binary digits, then its exact square root truncated toward zero to 24
significant binary digits. This is the function that approved S-0105 (F1) uses, and that S-0204
(coordinates, distance and facing) gives in exact integer form (its F1)
and analyses (its claim 5); whether our engine uses it is S-0204's switch
`quirks.table_root_distance` (default original), and this spec adds none. ρ is never more than ⌊√*n*⌋. For *n* below 536 987 929 (distances
below 23 173 leptons) it is never less than ⌊√*n*⌋ − 1; at that *n* it first falls 2 short, and
for larger *n* it can fall further (for example *n* = 40 001², a 2-D distance of 40 001 leptons
along one axis, gives ρ = 39 999). The leash of F5 measures from the actor to
its guard spot with the 3-D form and subtracts the building reduction β = 64 × (*W* + *H*) leptons
when the guard spot is a building with a *W* × *H* foundation, never going below 0. The guard spot's
reference point is that of its object; for a guard spot that is a cell it is the cell's centre at
the cell's height (confidence low, Open questions). The close-range test of F3 and the retaliation
reach of F7 use the plain ρ₂ and ρ₃ on the two positions, without the building reduction.

**F5. Area-guard leash.** Let *g* be the type's guard range in leptons (Import mapping) if it is not
0, else the larger of the ranges of the actor's primary and secondary weapons in leptons. Let
*G* = min(max(2*g*, 0), 4096) leptons. Then

> *L* = trunc(RZ₅₃(*G* × 1.1)) = ⌊11 × *G* / 10⌋ leptons

with 1.1 the double nearest to 1.1. The product is formed in floating point in the original; for
every integer *G* from 0 to 4096 it equals the exact integer form on the right (checked for all
4097 values with exact arithmetic). So *L* is at most 4505 leptons. When the guard spot is a mobile
actor (an actor guarding another unit), *L* is instead the game-wide guard stray distance in
leptons (Import mapping), with no factor and no limit; when no rules layer gives it, our engine
uses the first form of *L* instead (claim 36 b, a departure). The actor drops its target and returns when
the distance of F4 (with the building reduction) is greater than *L*.

**F6. Targeting timer.** On every automatic search covered by S-0309 claim 4, the targeting
timer is set to *N* + *U*(0, 2) ticks, where *N* is the area-guard targeting delay (current
behaviour *area guard*) or the normal targeting delay (otherwise), both integers in ticks.
The charging break-off search of S-0518 claim 15 leaves this timer unchanged and takes no
targeting-timer draw (claim 33). Apart from searches, a holder that drops its target by the drops
of S-0309 claims 27 and 28 with more than 10 ticks left on this timer resets it with one ranged
draw *U*(4, 8) ticks, inside the action that drops it, not at a search. With at most 10 ticks
left it takes no draw. No holder of any kind takes such a draw during game set-up (S-0309
claim 27, S-0501 claim 37). Each case of this reset has its owner's backing and level. At the drops
of S-0309 claim 27 that S-0309 scenario A18 runs, it is `measurable` by A18 part d, medium. At
the drops that S-0309 claim 27 rates `unmeasured`, among them every drop by a building or
aircraft holder (a building's pending delayed shot included), the reset is `unmeasured` with
them, since no part of S-0309 scenario A18 or of scenario B7 here runs those drops. Medium. At
the cloak drops and the underground drop of S-0309 claim 28 it is `unmeasured`, as S-0309 claim
28 rates it: no scenario there records the holder's targeting timer at those drops. Medium. The
set-up exception is `unmeasured`, as S-0309 claim 27 rates it: no scenario here makes a holder
lose its target during game set-up. Medium.

**F7. Retaliation reach for human-controlled owners.** The actor overrides to *attack* against an
attacker it cannot reach if ρ₃(actor, attacker) ≤ (*S* + 0.5) × 256, *S* the type's sight in cells
(an integer), the right side computed in double precision and compared with the integer ρ₃. Units:
leptons on both sides. The right side equals 256*S* + 128 exactly for every integer *S*, so no
rounding arises.

**F8. Turret snap bound** (claim 32). All values are integers in facing units (65 536 to a full
turn). Let *v* be the building type's turn-rate field and *w*₈ the integer in −128 … 127 congruent
to *v* modulo 256. Then

> *b* = |256 × *w*₈|  (so 0 ≤ *b* ≤ 32 768)
>
> *ε* = |*e*|, with *e* the integer in −32 768 … 32 767 congruent to *ψ*<sub>t</sub> − *ψ* modulo 65 536

where *ψ* is the turret facing and *ψ*<sub>t</sub> the direction to the target, both in facing
units. The snap happens when *v* = 0 or *ε* ≤ *b*. Exact integer arithmetic; no rounding arises.
For example *v* = 5 gives *b* = 1280, and *v* = 128 gives *w*₈ = −128 and *b* = 32 768, which every
*ε* meets. A field that is a multiple of 256 other than 0 gives *b* = 0, so only an exact aim
snaps. That the bound wraps modulo 256, where the turn rate of S-0206 clamps, is a quirk with a
proposed switch, `quirks.turret_snap_bound_wraps` (Edge cases and quirks).

**F9. Gate times** (claim 35). Let *t*<sub>d</sub> be the gate type's deploy time and
*t*<sub>c</sub> its gate close delay, both in game minutes as parsed by S-0003's decimal form
(Import mapping). Then

> *p*<sub>d</sub> = RZ₅₃(*t*<sub>d</sub> × 900),  *p*<sub>c</sub> = RZ₅₃(*t*<sub>c</sub> × 900)
>
> *T*<sub>d</sub> = *w*(trunc(*p*<sub>d</sub>)) ticks,  *T*<sub>c</sub> = *w*(trunc(*p*<sub>c</sub>)) ticks, each 0 instead when |*p*| ≥ 2⁶³ or *p* is not a number

each one double-precision multiplication, rounded as in F1 (toward zero with `quirks.chop_rounding`
on), then converted to a signed 32-bit integer exactly as F1 converts *p*: truncated toward zero
and wrapped by *w* of F1, or 0 when the product is too large for that conversion or not a number
(F1 says which values reach that case; a gate time of about 1.02 × 10¹⁶ game minutes or more gives
0). A negative result behaves as 0 (the door is at rest at its first
update; the countdown has run out at once).

**F10. Approach radius and renewal threshold** (claim 37). Let ρ be the firer's weapon range of
S-0307 claim 13 for the weapon chosen for the target, in leptons (integer). Then

> *r*′ = ρ − 128 if ρ ≥ 512, and *r*′ = ρ otherwise
>
> *r*′ = min(*r*′, ρ₃ + 512) for the deploy-to-fire vehicle of claim 37 f2 (ρ₃ its 3-D distance to the target, F4)
>
> the destination is cleared when ρ₃(target, destination) > *m* × *r*′

where ρ₃(target, destination) is F4's 3-D distance between the target's reference point and the
destination's reference point (the centre of a cell destination, or the object's reference point),
*m* is the approach reset multiplier (integer, Import mapping), and *m* × *r*′ is a signed 32-bit
integer product (reduced modulo 2³² into the signed range when it overflows, which only bad data
reaches). The comparison is strict. *r*′ is the ρ′ of S-0307's F4, so for the vehicle of claim 37
f2 the replaced value also starts that search (before S-0307 adds the building lengthening, which
the renewal threshold never includes).

**F11. House-selected centre** (claim 38). Let *P* = (*X*, *Y*, *Z*) be the preferred
recorded cell's reference point, or (0, 0, 0) leptons when neither cell exists. Let
*r* = *U*(0, *R*) in leptons, *b* be the subsequent raw value mod 256, and
*f* = 256*b* facing units. Form the point

> θ = (*f̂* − 16383) × (−9.587672516830327 × 10⁻⁵) radians
>
> *A* = trunc(*X* + *r* × cosT θ),  *B* = trunc(*Y* − *r* × sinT θ),  *C* = *Z*

*f̂* is *f* reduced into −32768 … 32767, exactly. This is the same reverse-facing
scale and floating-point grouping as S-0208 F3, with distance *r* instead of speed.
The angle multiplier is the double specified there. Each floating-point operation
is rounded to double under S-0205's rounding rule before the next, and trunc converts
to an integer toward zero. sinT and cosT are S-0105 F1's table trigonometry.
The shared switches `quirks.table_direction`, `quirks.table_trig_negative_bias` and
`quirks.chop_rounding` have their owning specs' alternatives here too.

If either trunc(*A* / 256) or trunc(*B* / 256) is outside 0 … 511, the point is
*P* instead. The retained point is snapped horizontally to
(256⌊*A* / 256⌋ + 128, 256⌊*B* / 256⌋ + 128), keeping its height.
The selected cell has coordinates trunc of each snapped coordinate divided by 256,
not floor division. In particular a snapped coordinate of −128 leptons selects
coordinate 0. Each snapped world coordinate is congruent to 128 modulo 256, so
the point cannot equal the empty world point (0, 0, 0).
All integer divisions here state their rounding explicitly; snapping uses floor,
whereas conversion to cell coordinates truncates toward zero.

For usable-area correction let (*x*, *y*) be that cell, *W* the map width, and
(*Lx*, *Ly*, *Lw*, *Lh*) the local area, in S-0009's units. A cell already passing
S-0009's height-aware usable test is unchanged. Otherwise define *d* = *x* − *y*
and the integer displacement *a* by this table:

| Condition | *a*, in cells |
|---|---|
| *d* ≥ 2(*Lx* + *Lw*) − *W* | −trunc((2(1 − *Lx* − *Lw*) + *W* + *d*) / 2) |
| *d* ≤ 2*Lx* − *W* | trunc((2*Lx* + 2 − *W* − *d*) / 2) |
| otherwise | 0 |

The lateral candidate is (*u*, *v*) = (*x* + *a*, *y* − *a*). At this candidate
let *h* be the terrain height level, plus 1 when it is sloped and
*u* + *v* < *W* + 4 + 2*Ly* + its height level. For a non-legal candidate use
S-0009 claim 44's stand-in cell for those terrain properties. The corrected centre is:

| Lateral candidate | Result |
|---|---|
| *u* + *v* ≤ *W* + 2*Ly* + *h* | (*u* + *k*, *v* + *k*), the smallest positive integer *k* for which that cell passes the height-aware usable test |
| *u* + *v* > *W* + 2 + 2(*Ly* + *Lh*) + *h* | (*u* − *k*, *v* − *k*), the smallest positive integer *k* passing that test |
| otherwise | (*u*, *v*) |

These corrections use exact integer arithmetic on supported maps. They are not a
nearest-Euclidean-cell projection: the result follows a diagonal and each tested
cell's own height-aware boundary. No randomness is added by correction.

**F12. House base-record refresh** (claim 38). Let *N* be the house's number of owned
buildings, and *B* its on-map buildings with positive health. For each such building
let *p*ᵢ be its type's current purchase price for this house in credits (S-0404 claim 2),
*w*ᵢ = max(0, trunc(*p*ᵢ / 1000) + 1), and (*X*ᵢ, *Y*ᵢ, *Z*ᵢ) its reference point.
Let *Q* = Σ*w*ᵢ. With *N* > 0 and *Q* > 0 the mean cell before the nearby search is

> (trunc(trunc(Σ(*w*ᵢ × *X*ᵢ) / *Q*) / 256),
>  trunc(trunc(Σ(*w*ᵢ × *Y*ᵢ) / *Q*) / 256))

The mean's two cell coordinates are each reduced modulo 65536 into
−32768 … 32767, as in S-0009's cell-coordinate conversion. With *N* = 0, or
*Q* ≤ 0, the base-start cell stays absent. The preferred recorded
cell after the search supplies reference point *P* for the radius; when absent,
*P* is (0, 0, 0) leptons. The new recorded radius is 0 when *N* = 0, 512 leptons
when *N* > 0 and *Q* ≤ 1, and otherwise

> radius = max(512, trunc(Σ distance₃(*P*, (*X*ᵢ, *Y*ᵢ, *Z*ᵢ)) / *Q*)) leptons

The distance sum includes each member of *B* once, not *w*ᵢ times: the price weights
only the mean and the denominator. distance₃ is F4's table-root distance truncated
toward zero. Integer sums and products in this refresh wrap as signed 32-bit values
(the wrap defined in F0); all displayed divisions truncate toward zero. Price's own
floating-point operations belong to S-0404; distance's belong to F4. The remaining
arithmetic is integer arithmetic. The minimum 512 leptons of this record is separate
from the 768-lepton minimum used by the later random choice.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Default processing delay | 450 | ticks | Claims 4 and 30; no draw |
| Ticks per game minute | 900 | ticks per game minute | F1; glossary *game minute* |
| Rate absent-key default | double nearest 0.016 | game minutes | Every behaviour; gives *D*<sub>r</sub> = 14 |
| Anti-air rate absent-key default | the rate | game minutes | Claim 21 |
| Flag absent-key defaults | no-threat no, paralysed no, recruitable yes, retaliate yes, scatter yes, zombie no | dimensionless (yes/no) | Every behaviour |
| Approach range cut and reduction | 512 (inclusive), 128 | leptons | F10; the same values as S-0307 F4 |
| Deploy-to-fire approach margin | 512 | leptons | F10, claim 37 f2 |
| Approach reset multiplier absent-key default | 1 | dimensionless (integer) | F10 |
| Can-approach and can-recalculate absent-key defaults | yes, yes | dimensionless (yes/no) | Claim 37 d and e |
| Standard draw range | 0 to 2 | ticks | F2 |
| Area-guard draw range | 1 to 5 | ticks | Claim 26 |
| Area-guard harvester draw range | 1 to 10 | ticks | Claim 26; plus 1 |
| Deploying-infantry draw range | 10 to 20 | ticks | Claim 31; added to *D*<sub>r</sub> |
| Radiation threshold divisor | 3 | dimensionless | Claim 31; *R* = trunc(*W* / 3) |
| Attack close-range divisor | 2 | dimensionless | Claim 23, F3 |
| Area-guard close-range divisor | 6 | dimensionless | Claim 26, F3 |
| Area-guard rate factor *k* | 2 for aircraft, 1 otherwise | dimensionless | Claim 26, F3 |
| Unarmed building rate factor | 3 | dimensionless | Claim 32; 3 × *D*<sub>r</sub> + *U*(0, 2) |
| Ticks per game minute for gate times | 900 | ticks per game minute | F9 |
| Delay of *open* for a building that is not a gate | 1 | ticks | Claim 35 (c); no draw |
| Delay of a gate's *open* stages other than Std | 0 | ticks | Claim 35 (c); no draw |
| Absent-key default: area-guard threshold | 4 | house skill levels | Claims 14 and 15 |
| Absent-key defaults: gate flag, deploy time, gate close delay | no, 0, 0 | boolean, game minutes, game minutes | Claim 35, F9 |
| Close-range band | 282 to 768 | leptons | F3, 2-D distance |
| Close-range weapon threshold | 512 | leptons | F3, primary weapon range at most this |
| Leash cap before the factor | 4096 | leptons | F5, *G* |
| Leash factor | 1.1 (double nearest) | dimensionless | F5; *L* = ⌊11*G*/10⌋, at most 4505 leptons |
| Building reduction per foundation cell of width and height | 64 | leptons | F4, leash only |
| Factory exit cell | (0, +1) from the foundation's top-left cell | cells | Claim 11 |
| Unarmed stupid-guard building delay | 100 | ticks | Claim 32; no draw |
| Non-SAM attacking building delay | 1 or 2 | ticks | Claim 32; 2 when not facing the target or rearming, else 1; no draw |
| SAM attacking building delay | 1 | ticks | Claim 32; no draw. Exception: the out-of-stage rule chooses the anti-air delay *D*<sub>aa</sub> + *U*(0, 2) |
| SAM aiming tolerance | 8192 | facing units | Claim 32 (45 degrees) |
| Turret snap bound step | 256 per unit of *w*₈ | facing units | Claim 32, F8; *w*₈ the integer in −128 … 127 congruent to the turn-rate field modulo 256 |
| Turret snap bound maximum | 32 768 | facing units | F8 (half a turn) |
| Normal targeting delay absent-key default | 27 | ticks | F6, engine default |
| Area-guard targeting delay absent-key default | 36 | ticks | F6, engine default |
| Human-controlled house skill level in a skirmish | 0 | house skill level | Claim 15; also every AI house of our engine and the neutral and special houses in a skirmish (S-0412, claim 3) |
| Charge-target search | 8 | cells | Claims 24, 26: the neighbouring cells |

**T1. Behaviours and which actor kinds have a processing rule.** "—" means the default processing
of claim 4 (nothing, 450 ticks). "own" means a rule of its own, described in the claim named or owned
by the topic named in claim 34. "as X" means the processing of behaviour X, with the
properties (rate, anti-air rate) of the behaviour that is current (claim 5).

| Behaviour | Infantry | Vehicles | Aircraft | Buildings | Claim |
|---|---|---|---|---|---|
| sleep, harmless, ambush, return, stop, wait | — | — | — | — | 4 |
| none (no current behaviour) | — | — | — | — | 4 |
| queued move, attack move | — | — | — | — | 5 |
| attack | own | own | own | own | 23, 32, 34 |
| move | own | own | own | — | 22, 34 |
| guard | own | own | own | own | 24, 31, 32, 34 |
| sticky | as guard | as guard | as guard | as guard | 25, 32 |
| area guard | own | own | own | as guard | 26, 31, 32 |
| hunt | own | own | own | — | 27, 34 |
| capture | own | own | own | — | 28 |
| sabotage | as capture | as capture | as capture | — | 28 |
| eaten | own | own | own | — | 28 |
| enter | own | own | own | — | 29, 34 |
| unload | own | own | own | own | 30, 34 |
| retreat | own | own | own | — | 34 |
| rescue | own | own | own | — | 34 |
| patrol | own | own | own | — | 34 |
| harvest | own | own | — | — | 34 |
| construction, selling, missile | — | — | — | own | 34 |
| open | — | — | — | own | 35 |
| repair | — | own | — | own | 34 |
| paradrop approach, paradrop overfly, spy-plane approach, spy-plane overfly | — | — | own | — | 34 |

**T2. Delays and draws of the general-purpose processings** (mobile actors), plus the infantry rows
of claims 23 and 27; see the claims for the other kind-specific rules. "Std" is the standard delay *D*<sub>r</sub> + *U*(0, 2). The draw column counts
draws of the processing itself; a target search it runs adds the search's own draws. A search
covered by S-0309 claim 4 adds one *U*(0, 2) for the targeting timer (claim 33); the charging
break-off search of S-0518 claim 15 adds no targeting-timer draw. Candidate draws are those
S-0309 gives in its claim 12: none for an actor of a player house, and the disguise draws of its
claim 25, part g, for the neutral and special houses. The targeting-timer reset of F6 at a drop
of S-0309 claims 27 and 28 is taken inside the action that drops the target, so it is no draw of
a processing here.

| Behaviour | Outcome | Delay (ticks) | Draws |
|---|---|---|---|
| none, sleep, harmless, ambush, return, stop, wait, queued move, attack move | always | 450 | 0 |
| move | no destination, not moving, nothing queued | 1 | 0 |
| move | otherwise | Std | 1 |
| attack | general rule | Std, or trunc(Std / 2) at close range | 1 |
| attack (infantry) | explosive-charges or computer capture row of claim 23 | 1 | 0 |
| attack (infantry) | deployed, human-controlled owner | Std | 1 |
| guard, sticky | holding an enter-nearest instruction (in our engine only the absorbing-building one) | *D*<sub>r</sub> (of *capture* if an occupy instruction started it) | 0 |
| guard, sticky | weapon rearming | rearm time remaining | 0 |
| guard, sticky | distributed-fire type with candidates | 0 | 0 |
| guard, sticky | otherwise | Std | 1 |
| guard, area guard (deploying infantry) | the rows of claim 31 | as claim 31 | 0 or 1 |
| area guard | slave master's wake-up rule (S-0402, claim 14) holds (vehicles, row 1) | Std | 1 |
| area guard | in a bunker; or sabotage queued (original only) | 1 | 0 |
| area guard | holding an enter-nearest instruction (in our engine only the absorbing-building one) | *D*<sub>r</sub> (of *capture* if an occupy instruction started it) | 0 |
| area guard | harvester (vehicles) | 1 + *U*(1, 10) | 1 |
| area guard | search found a target | 1 | 0 |
| area guard | otherwise | *k* × *D*<sub>r</sub> + *U*(1, 5), or trunc(that / 6) at close range | 1 |
| hunt | general rule | Std | 1 |
| hunt (infantry, computer-controlled owner; original only) | the capture or undeploy rule of claim 27 (no search) | 1 | 0 |
| capture, sabotage, eaten | always | Std | 1 |
| unload (deploying infantry) | as the table of claim 30 | as claim 30 | 0 |

Where a processing makes another behaviour current before choosing a delay from a rate (the
general *hunt* row, and the enter-nearest rows when an occupy instruction's immediate start makes
*capture* current), the rate is that of the behaviour it made current (claim 6).

The delay value uses the stream position after the assignments to the processing's other draws
(claim 6, observable random-stream result).

## State and ordering

**Slot transitions** (claims 7 to 10). "Q" is the queued, "C" the current and "S" the suspended
behaviour. For each event, the first row that matches applies (as in claim 7).

| Event | Condition | Result |
|---|---|---|
| queue *b* | the actor is an aircraft other than an airstrike aircraft, C is *retreat*, either paradrop behaviour or either spy-plane behaviour, and *b* is none of those five (S-0501 claim 34) | nothing |
| queue *b* | C is *selling*, or C is *wait* and *b* is *guard* | nothing |
| queue *b* | *b* is none | nothing |
| queue *b* | *b* is C, and Q is none or *b* | nothing |
| queue *b* | any other case (*b* differs from C, or Q holds a behaviour other than *b*) | *b* becomes Q, replacing any queued behaviour |
| start-next | Q ≠ none | the behaviour in Q becomes C and Q becomes none; S unchanged (`quirks.stale_suspended_revert`); timer expired now; start tick now; progress count 0 |
| start-next | Q = none | nothing |
| force *b* | C is *wait* and *b* is *guard* | nothing |
| force *b* | otherwise (C *selling* included) | *b* becomes C and Q becomes none; S unchanged (`quirks.stale_suspended_revert`); timer expired now; start tick now; progress count 0 |
| override *b* | C is *selling*, or C is *wait* and *b* is *guard* | no slot change; target and destination still put aside and replaced (`quirks.refused_override_retargets`) |
| override *b* | otherwise | S becomes the queued behaviour if Q ≠ none, else the behaviour that was current; *b* becomes C; Q unchanged; target and destination put aside and replaced |
| revert | S ≠ none | the behaviour in S becomes C and S becomes none; the put-aside target and destination restored (none, after a behaviour order since the override, claim 12) |
| revert | S = none | nothing |

The queue rows show the slots only. When the request asks for an immediate start, a start-next
follows as claim 7 says in every queue row except the first two, whose requests are ignored
entirely.

**Typical behaviour transitions** (consequences of the claims, for a mobile actor of a
human-controlled owner):

| Current | Event | Result |
|---|---|---|
| any | move order | *move* queued; current at the next per-tick start when ready (claim 12) |
| *move* | due processing finds no destination, no movement and nothing queued | falls back to rest: usually *guard* or *area guard* queued, then current at the next permitted start, including that update's late start if ready (claims 11, 14, 15, 22) |
| *move* (infantry) | at the end of an update, still current, not moving and no destination, after its arrival, a stop order, a failed reachability gate or a teleporting unit's jump | falls back to rest by claims 13 and 14: usually the rest behaviour queued, then current at the next first start if ready, processed at once (claim 22) |
| *move* | stop order | no slot change; destination cleared; falls back to rest at a processing after it has stopped (claims 12, 22); infantry at the end of the update after which it has stopped, unless a *move* processing already finds it stopped (claim 22) |
| *attack* | target gone | falls back to rest (claim 23) |
| *guard* | attacked, retaliation allowed | override to *attack*; revert to *guard* at the killing hit on the attacker, or when the attacker goes off the map or changes owner (claims 10, 18) |
| any but *wait* | taken over by mind control, with the guard reset applicable under S-0601 claim 6 | *guard* current at once; target, destination and guard spot cleared (claim 9) |
| *wait* | taken over by mind control | *wait* stays current (the forced *guard* is refused); target, destination and guard spot cleared (claim 9; scenario B3, case i) |
| *unload*, vehicle that deploys in place | taken over by mind control | no takeover guard reset; S-0601 claim 6 owns the exception and claim 9 owns release |
| *area guard* | target leads it beyond the leash | target dropped, returns to the guard spot (claim 26) |
| *hunt* (skirmish) | no target found | stays in *hunt*, idles, searches again next processing (claim 27) |
| a behaviour with the no-threat property but not zombie, mobile actor not in a team | a hit from a known attacker that lowers its health and leaves it alive (claim 16) | falls back to rest (claim 16) |
| *sticky* | behaviour order | queued but never started while *sticky* lasts (claim 25); stop has claim 12's separate effects |
| a behaviour with the zombie or paralysed property | falls back to rest with no target, destination or queued destinations (infantry), or as an armed vehicle that is not a harvester, with no destination or queued destinations | the behaviour stays current and nothing is queued (claim 17) |

**Ordering within a tick, as observed.** This list only gathers, for one infantry unit, vehicle or
building on one tick, the order statements that the claims named in each item make and back; it
adds none of its own:
- a behaviour that the per-tick start makes current (for a building, its first per-tick start of
  the tick) is the one processed on that tick (claim 3a), and the one whose drop list applies on
  that tick (claim 33, part a),
- a processing never acts on an opportunity target that the drop rule removes on the same tick
  (claim 33, part a),
- a target that the per-tick search finds on a tick is first acted on by the processing of a later
  tick, and is not removed by the drop rule of the same tick (claim 33, part c),
- a target found by the per-tick search can still be approached or dropped by the later
  per-tick approach on that same tick; the approach judges the state after processing and
  search, on processing and non-processing ticks alike, and is not suppressed merely because
  processing already took an approach step (claim 37 a2),
- whether the per-tick search runs is decided by the current behaviour as that tick's processing
  leaves it (claim 33, part b),
- the processing's choices receive earlier stream values than the per-tick search's choices
  on the same tick (claim 33, part d); the processing's delay receives the value after its other
  draw assignments (claim 6). These constrain outcomes and resulting stream state, not internal
  execution order,
- for an actor that is ready to change, a behaviour order's behaviour becomes current and is
  processed at the actor's first update after the order takes effect (claim 12; in a single-player
  game that is its update on the next tick, S-0203, claim 7),
- for a building, a queued behaviour can also become current after the building's processing and
  per-tick search on the same tick (its second per-tick start, claim 11); it is then first
  processed on the next tick (claim 3b), and
- for an infantry unit or vehicle, a queued behaviour can also become current after its processing,
  per-tick search and movement on the same tick (its late start, claim 11); it is then first
  processed on the unit's next update (claim 3b), and neither the drop rule nor the per-tick search
  of the tick that made it current ever sees it as current (claim 11, claim 33 part b).

Where orders take effect within the tick belongs to S-0203 (tick order) and S-0213 (human
orders).

## Edge cases and quirks

- **Queued move and attack move do nothing** (claim 5). The original processes both as *sleep*. A
  unit put into either does nothing by itself (it does not even search for targets): only orders,
  retaliation (claim 18) and other rules can make it act. This includes a unit that an attack-move
  order without target and destination put into *attack move* (claim 12): it stands still. Proposed
  switch `quirks.queued_move_sleeps` (default on = original). With it off, both are processed
  exactly as *move* (claim 22), and both are added to the behaviours that run the per-tick
  opportunity search of claim 33, under the same passive-search conditions as *move*.
- **Override forgets the current behaviour when one is queued** (claim 10). If a behaviour is queued
  at the moment of an override, that queued behaviour, not the current one, is put aside, and it also
  stays queued; the behaviour that was current is lost. Proposed switch
  `quirks.override_suspends_queued` (default on = original). With it off, the override always
  suspends the current behaviour and leaves the queue as it was.
- **A refused override still changes the target** (claim 10). An override on an actor in *selling*,
  or an override to *guard* on an actor in *wait*, leaves the behaviour slots alone but still puts
  the target and destination aside and replaces them. Proposed switch
  `quirks.refused_override_retargets` (default on = original). With it off, a refused override
  changes nothing.
- **Anti-air rate reset by a layer** (claim 21). A rules layer that restates a behaviour's section
  without an anti-air rate replaces any anti-air rate an earlier layer gave with the rate. Proposed
  switch `quirks.anti_air_rate_layer_reset` (default on = original). With it off, an absent anti-air
  rate keeps the held value (S-0006, claim 3), and only a behaviour whose anti-air rate was never
  given, or is given as 0, uses the rate.
- **Rates and delays that do not fit.** If *r* × 900 is 2³¹ or more (or below −2³¹), the rate delay
  wraps as F1 states, and it is 0 when the product's magnitude is 2⁶³ or more; sums and products formed from it wrap as F0 states, and a negative delay means
  processing on every tick (claim 2). Our engine reproduces this. The importer warns for any rate
  whose |*r* × 900| (F1, before the conversion) is above 715 827 875 ticks, and for any negative rate,
  so a rate whose delay wraps, or becomes 0 at 2⁶³ or more, is warned about although its
  *D*<sub>r</sub> is small. That bound is
  conservative: no delay of this spec can wrap at or below it, and the first that can,
  3 × *D*<sub>r</sub> + 2, wraps only for *D*<sub>r</sub> above 715 827 881 ticks. This is a consequence
  of the integer domain of F0, not a quirk of its own, and it has no switch of its own here.
- **Rate precision** (claim 6, F1). The one-tick shortfall of rates written as short decimals
  without a percent sign is a consequence of single-precision parsing (S-0003) alone: the product
  with 900 is exact (S-0205, claim 9), so `quirks.chop_rounding` does not change it. For a rate
  written with a percent sign the product is not exact and `quirks.chop_rounding` decides it (`6%`
  gives 53 ticks on, 54 off; S-0205 effects table). The shortfall has no switch of its own here.
- **Routes are looked up in the local player's list** (claim 13, whose route). When an actor that follows a
  route falls back to rest, the original finds the route's next point in the route list of the
  house played on the machine running the game, not in that of the actor's owner, so the result
  depends on which machine computes it and is unspecified on the others. Our engine uses the
  owner's routes on every machine and judges repair/reload route-point alliance guards and
  the non-allied fallback from the actor owner's house (departure, no switch). There is no
  machine-local house in the simulation.
- **Auto-deploy delay list too short** (claim 31). If the game-wide auto-deploy delay list has no
  entry for an owner's difficulty index, the original's value for that index is unspecified
  (claim 31; Terms). In our engine only the units of the neutral and special houses reach the
  auto-deploy row, and both have difficulty index 1, so only the list's second entry is read. For
  a list without a second entry our engine applies no auto-deploy to those houses (claim 31,
  documented departure, no switch). The importer warns when the list has fewer than 3 entries,
  so that an AI that reproduces the original's auto-deploy (claim 31, note for AI designers) does
  not rely on a short list. Such an AI should then use the last entry, or no auto-deploy if the list
  is empty.
- **A rule tied to one type name** (claim 30). The original picks out the infantry type of claim
  30's area-fire rule by a type name fixed in its program (Import mapping), not by a data field, so
  a mod that renames that type loses the rule and a mod that gives another type that name gains
  it. Proposed switch `quirks.named_deploy_area_fire` (default on = original). With it off, no
  type is singled out by name: a unit of that type is treated like any other type in claim 30, so
  when it deploys with an area-fire deploy weapon it takes its own cell as its target, and its
  delay is the one claim 30's table gives for "any other type" (450 ticks when its undeploy delay
  is negative, else the undeploy delay). The switch changes nothing else.
- **A reopening gate door snaps open** (claim 35). A gate door that reverses from closing to
  opening reaches rest open at the gate's next update (claim 35 d), however far it had closed. Proposed
  switch `quirks.gate_reverse_snaps` (default on = original). With it off, the reopening takes as
  many ticks as the closing had run until the reversal, so the door retraces its path.
- **The turret snap bound reads the turn-rate field modulo 256** (claim 32, F8). The snap bound
  of a turreted building takes the turn-rate field as a signed value modulo 256, unlike the turn
  rate itself, which S-0206 (claim 6) clamps at 127. So a field of 128 snaps at any aim error, a
  field of 200 snaps at errors up to 14 336 facing units although the turret turns at the rate of
  127, and a non-zero multiple of 256 snaps only at an exact aim. Fields from 0 to 127 are not
  affected. Proposed switch `quirks.turret_snap_bound_wraps` (default on = original). With it
  off, the bound is *b* = 256 × ω, with ω the turn rate that S-0206 (claim 6, F3) gives for the
  field, and the snap happens when ω = 0 or *ε* ≤ *b*. Scenario B16 (h) observes the original's
  side with a field of 128. The importer warns for a turreted building
  type whose turn-rate field is outside 0 to 127, where the two rules differ (S-0206 already warns
  for a negative field).
- **No guard stray distance in any layer** (claim 36 b). When no rules layer gives the guard stray
  distance, the original's leash for a guard spot that is a mobile actor is unspecified. Our
  engine uses the area-guard leash of F5, as for any other guard spot, and the importer warns
  (documented departure by the project lead's delegated decision of 2026-10-10, no switch, since
  the original's value cannot be reproduced).
- **A late revert can bring back an old behaviour** (claims 8 to 10 and 12). A start-next, a force
  and a behaviour order all leave the suspended slot as it is, and an override stays in effect
  while a behaviour is suspended (claim 10). So an override that none of claim 10's causes has
  ended (for example a retaliation interrupted by a move order) keeps its suspended behaviour, and
  a later revert, caused by whatever object is then the actor's target, makes that behaviour
  current again, possibly long after the override. That revert brings back the target and
  destination still put aside: after a behaviour order there are none (claim 12), so the actor
  then has no target and, if mobile, no destination. Example: a vehicle in *guard* retaliates, is
  ordered to move, and is then ordered to attack another enemy; at the killing hit on that enemy
  *guard* becomes current by a revert, not by the fall-back of claim 23. Had the vehicle taken an
  opportunity target during its move (S-0309), the killing hit on that target would stop it short
  of its destination. Proposed switch `quirks.stale_suspended_revert` (default on = original).
  With it off, every start-next that makes a queued behaviour current and every force that takes
  effect, whatever makes it (a per-tick start, an immediate start asked for by a queue request or
  made by a processing, a guard reset or any other force), also empties the suspended slot and
  forgets the put-aside target and destination, so a later revert finds nothing suspended and
  does nothing. A start-next with nothing queued and a refused force (claims 8 and 9) still do
  nothing. An override and a revert with no
  start-next or force between them behave as with the switch on. With the switch off and
  `quirks.override_suspends_queued` on, an override made while a behaviour is queued therefore
  ends when that queued behaviour starts. Scenario B7 (h, l) observes the original's side.

## Import mapping

Q-0161 adds no imported timing setting. Claim 37 a2's placement and independent eligibility
are fixed rules, not behaviour-rate or targeting-delay fields.

| Data input | Concept | Parse form and absent-key result |
|---|---|---|
| none | route ownership and alliance perspective (claim 13, Q-0663) | fixed owner-routes departure, including both service alliance guards and non-allied fallback; no imported local-house setting or absent-key choice |
| none | driving readiness, retained-goal waits and first start on the body-turn completion tick (claim 11, Q-1160, Q-1161) | fixed runtime predicate, no new key or absent-key default; body turn duration and integer movement step use S-0206 claims 1 and 6 and S-0207 claim 2, with their existing imports |
| none | per-tick approach placement and independent eligibility (claim 37 a2) | not configurable through data; fixed as stated in that claim |
| none | same-processing qualification of a newly taken own-cell guard spot, search and idle-action outcomes (claim 26, Q-0231) | fixed behaviour, no imported setting or absent-key choice; the existing behaviour-rate and targeting-delay fields still supply their delays |
| none | ready mark from the grand opening at set-up placement and *guard* current from the first per-tick start (claim 1, Q-0241) | fixed behaviour, not a new data flag; it does not depend on any weapon, turret or upgrade field. Building map entry, upgrade, idle body and turret flag mappings stay with S-0009, S-0108 and the rows below |
| none | house-centre selection after path give-up (claim 38, Q-0238, Q-0312) | fixed behaviour and constants, F11–F12; no per-behaviour rate or new setting |
| building type `Cost` and its price factors | price weight of base-record refresh (claim 38, F12) | integer cost and price-factor parsing/defaults stay with S-0403, S-0404 and S-0413; use the current purchase price, not a copied stock cost |
| map geometry and start assignment | usable-area correction and recorded base-start cell (claim 38) | parse forms and absent-key rules stay with S-0009; no implicit unit-relative centre |

**Behaviour property sections.** Each behaviour's properties come from a game-wide section of the
rules data named after the behaviour. Section names are looked up exactly as S-0001 says (with
case), in the spelling of this table:

| Behaviour | Section | Behaviour | Section |
|---|---|---|---|
| ambush | `[Ambush]` | paradrop overfly | `[Paradrop Overfly]` |
| area guard | `[Area Guard]` (with a space) | patrol | `[Patrol]` |
| attack | `[Attack]` | queued move | `[QMove]` |
| attack move | `[Attack Move]` | repair | `[Repair]` |
| capture | `[Capture]` | rescue | `[Rescue]` |
| construction | `[Construction]` | retreat | `[Retreat]` |
| eaten | `[Eaten]` | return | `[Return]` |
| enter | `[Enter]` | sabotage | `[Sabotage]` |
| guard | `[Guard]` | selling | `[Selling]` |
| harmless | `[Harmless]` | sleep | `[Sleep]` |
| harvest | `[Harvest]` | spy-plane approach | `[Spyplane Approach]` |
| hunt | `[Hunt]` | spy-plane overfly | `[Spyplane Overfly]` |
| missile | `[Missile]` | stop | `[Stop]` |
| move | `[Move]` | sticky | `[Sticky]` |
| open | `[Open]` | unload | `[Unload]` |
| paradrop approach | `[Paradrop Approach]` | wait | `[Wait]` |

Keys in each section:

| Key | Property | Form | Absent key |
|---|---|---|---|
| `NoThreat` | no-threat | boolean (S-0003) | held value; engine default no |
| `Paralyzed` | paralysed | boolean | held value; default no |
| `Recruitable` | recruitable | boolean | held value; default yes |
| `Retaliate` | retaliate | boolean | held value; default yes |
| `Scatter` | scatter | boolean | held value; default yes |
| `Zombie` | zombie | boolean | held value; default no |
| `Rate` | rate, game minutes | decimal (S-0003 claims 6 and 7; `%` divides by 100) | held value; default the double nearest 0.016 |
| `AARate` | anti-air rate, game minutes | decimal, as `Rate` | becomes the rate (claim 21); an explicit 0 also becomes the rate |

The sections are read in every rules layer that contains them (S-0006); a layer that does not
contain a section changes none of its values. They are game-wide sections, so their values carry
over between games of a session (S-0006, claim 10). A layer that contains the section resets
`AARate` as claim 21 says.

The same 32 names, matched without regard to case, are how data names a behaviour elsewhere, for
example for objects that a map places. A name that matches none of them means *none*. Which data
fields name behaviours belongs to the specs of those fields.

**Fields owned by this spec** (area-guard threshold, guard stray distance and gate times; S-0006 layering applies, so an
absent key keeps the value the earlier layers left).

| Key | Section | Feeds | Form | Absent key |
|---|---|---|---|---|
| `GuardArea` | `[IQ]` | area-guard threshold of claims 14 and 15 | integer (S-0003, claim 3) | held value; engine default 4 |
| `GuardModeStray` | `[General]` | guard stray distance, the leash when guarding a mobile actor (F5) | cells, in S-0003's decimal form (claims 6 and 7), converted to leptons as stated below the table; −1 also keeps the held value | held value; the original has no engine default; our engine then uses the area-guard leash of F5 and warns (claim 36 b) |
| `GateCloseDelay` | building type section | gate close delay *t*<sub>c</sub> (F9) | decimal, game minutes (S-0003, claim 6) | held value; engine default 0 |
| `DeployTime` | building type section (the key exists for every type kind; this spec reads it only for gates) | gate door time *t*<sub>d</sub> (F9) | decimal, game minutes | held value; engine default 0 |

**Other fields used by this spec** (owned by other specs; listed so the references above resolve).

| Key | Section | Use here | Form and absent key |
|---|---|---|---|
| `NormalTargetingDelay` | `[General]` | *N* of F6 | integer ticks; engine default 27 |
| `GuardAreaTargetingDelay` | `[General]` | *N* of F6 in *area guard* | integer ticks; engine default 36 |
| No data key | not applicable | claim 13's refresh priority, destination collisions and route-order timing (Q-0202) | fixed behaviour; memory is created by orders (S-0213), not an imported field; no absent-key default |
| `AIAutoDeployFrameDelay` | `[General]` | auto-deploy delay list *A* of claim 31 (in our engine read only for the units of the neutral and special houses, by their difficulty index 1) | decimal-prefix signed 32-bit integer list in ticks, S-0003 claim 22 and C20: no scalar hexadecimal form, unreadable non-empty items retain a place with 0, empty items are skipped, 511-byte text limit subject to the physical-line cut; present non-empty replaces the list, missing or empty keeps it; entries for indexes 0, 1 and 2 (Terms); engine starting default open (Open questions) |
| `PlayerReturnFire` | combat-damage settings section (S-0006) | lifts the behaviour restriction of claim 18 | boolean |
| `PlayerScatter` | combat-damage settings section (S-0006) | player-scatter setting of claim 19 (the scatter from damage; S-0208 claims 11 and 15 for infantry) | boolean |
| `MaxIQLevels` | `[IQ]` | house skill level of the houses of the original's computer players in a skirmish (claim 15; S-0412, claim 3); not used for AI houses of our engine, nor for the neutral and special houses, which keep 0 | integer |
| `BuildConst` | `[AI]` | construction-yard types for the vehicle rules of claims 24 and 27 (the original-only rules, and claim 27's deploy rule) | list of building types |
| `GuardRange` | type section | *g* of F5 | cells, in S-0003's decimal form (claims 6 and 7), converted to leptons as stated below the table; absent or −1 keeps the held value; engine default 0 |
| `DefaultToGuardArea` | type section | claims 14 and 15 | boolean |
| `Gate` | building type section | whether a building is a gate (claim 35) | owned by S-0209 (Import mapping) |
| `GateStages` | the building's image section in the art data (S-0007) | *G*, the gate's drawn stage count (claim 35 d1–d3); drawing only, no effect on passage timing | integer, stages; absent keeps the held value; engine default 9 stages; image selection owned by S-0108 claim 4 |
| `StupidHunt` | type section | claim 27 | boolean |
| `HoverAttack` | type section | claims 23 and 24 | boolean |
| `ApproachTargetResetMultiplier` | `[General]` | *m* of F10 (claim 37 e) | integer; absent keeps the held value; engine default 1 |
| `CanApproachTarget` | type section | the can-approach property (claim 37 d) | boolean; absent keeps the held value; engine default yes |
| a type boolean whose key is not yet on the allowed-key list (Questions for the project lead) | type section | the can-recalculate property (claim 37 e) | boolean; absent keeps the held value; engine default yes |
| `DeployToFire` | vehicle type section | the deploy-to-fire vehicle of claim 37 f2 (owned by S-0301) | boolean |
| `CloseRange` | infantry type section | F3 | boolean |
| `DistributedFire` | type section | claim 24 | boolean |
| `OpportunityFire` | type section | claim 33 (passive search while moving) | boolean |
| `Sight` | type section | F7 | integer cells |
| `C4` | infantry type section | explosive charges (claims 23, 24, 26, 27, 28) | boolean |
| `Engineer`, `Occupier`, `Assaulter`, `Infiltrate`, `VehicleThief` | infantry type section | claims 14, 23, 27, 28 | boolean |
| `Deployer`, `DeployFire`, `UndeployDelay` | type sections | claims 22, 30, 31 | boolean, boolean, integer ticks |
| `Harvester`, `Weeder` | vehicle type section | claims 15, 24, 26 | boolean |
| `DeploysInto` | vehicle type section | claims 15, 24, 27 | building type |
| `CanC4`, `InvisibleInGame` | building type section | claim 23 | boolean |
| `Overpowerable` | building type section | charge targets (claims 24, 26) | boolean |
| `InfantryAbsorb`, `UnitAbsorb` | building type section | absorbing buildings (Terms; claim 24) | boolean |
| `CanBeOccupied`, `TechLevel` | building type section | owned occupiable buildings (Terms; claim 24): the flag set and the tech level 0 or more; the flag also in claim 32 | boolean; integer, engine default 255 (so a type that gives no tech level counts) |
| `WeaponsFactory`, `Bunker` | building type section | claims 11, 15 | boolean |
| `ElectricAssault` | warhead section | the secondary weapon's warhead that marks a charging weapon (claims 24, 26) | boolean |
| `HasStupidGuardMode`, `UnitRepair`, `SAM`, `Powered`, `EMPulseCannon` | building type section | claim 32; `UnitRepair` also feeds claim 13's route service test, default no (S-0314) | boolean |
| `UnitReload` | building type section | claim 13's route reload test | boolean, default no (S-0501) |
| `Strength`, `Ammo` | actor type section | maximum health and maximum ammunition for claim 13's route tests | integer, defaults owned by S-0302 and S-0301 |
| `Power` | building type section | claim 32: the building drains power when the value is negative (the power rules belong to the structures topic) | integer |
| `ImmuneToRadiation` | type section | claim 31 | boolean |
| `RadLevel` | weapon section | *W* of claim 31 (of the secondary weapon) | integer |
| No data key | not applicable | claim 31's immediate secondary shot and same-processing own-cell target removal (Q-0983) | fixed behaviour, no absent-key default; no fire-frame field controls this shot |
| `DeployedFire` | art sequence section | claim 31's post-shot sequence and its frame-count processing delay | sequence entry, parsing and absent-entry behaviour owned by S-0107 claim 1 and its Import mapping; does not schedule this shot |
| `AreaFire` | weapon section | claim 30 (of the deploy weapon, Terms) | boolean |
| `Burst` | weapon section | upgrade weapon gate's burst-count comparisons (claim 32) | integer shots; engine default 1, owned by S-0301 claim 4 |
| `IsAnimDelayedFire`, `DelayedFireDelay` | art, building image section | ordinary animation-delayed firing, bypassed by an active passing upgrade gate (claim 32; S-0301 claim 14) | boolean, default no; integer ticks, default 0 |
| `Upgrades`, `PowersUpToLevel`, `PowersUpBuilding` | building type section | upgrade count, occupied positions and host eligibility, not installation-history precedence (claim 32) | integer, default 0; integer, default −1; building type, absent none (S-0406 claim 12). No new ordering key. |
| `SprayAttack` | type section | spray-attack flag: picks the first weapon slot as the deploy weapon (Terms; claim 30) | boolean; absent no |
| `Turret` | building type section (and the building types installed on it as upgrades) | turret flag of a turreted building (Terms; claim 32) | boolean; absent no |
| `TurretAnimIsVoxel` | rules, section named by the building type | voxel turret (Terms; claim 32; S-0310 claim 4 and S-0313 claim 10) | boolean; initially no; omission in a later rules layer keeps the current value; art supplies no value or fallback |
| `ROT` | building type section | turn-rate field *v* of F8 (claim 32) | integer; absent 0 |

Goal-cell cleanup in claims 22 and 29 introduces no imported field, delay or behaviour
property. The distinction is whether the assigned destination names a cell or a building.
Locomotion owns cleanup (S-0207 claim 26), and the entry fields retain S-0314 claims 16–18's
mapping.

**Cells to leptons** (`GuardRange` and `GuardModeStray`). The value is read as S-0003's decimal
form: a single-precision value widened exactly to double, or, with a percent sign, the
double-precision product of S-0003 claim 7. A value equal to −1 exactly keeps the held value.
Any other value *c* gives *w*(trunc(*c* × 256)) leptons, with *w* of F1, or 0 when
|*c* × 256| ≥ 2⁶³: the product is formed in double precision and is exact (256 is a power of
two), and it is converted to a signed 32-bit integer as F1 converts *p*.
For example 4.5 cells give 1152 leptons and 0.3 cells (held as 0.300000011920929) give 76.

The infantry type singled out by name in claim 30 is identified by a type name fixed in the original's program and compared without regard to case. The literal name is omitted here: a mod that renames that type loses the rule, and a mod that gives another type the hard-coded name gains it (`quirks.named_deploy_area_fire`, Edge cases and quirks).

The guard-area veterancy ability and the explosive-charges veterancy ability are entries of the
veterancy ability lists (veterancy specs).

## Measurement scenarios

Common setup: the measurement setup of the glossary, a small flat test map, skirmish, stock data
unless a scenario names a test file. No external automated player or state-changing engine add-on
is active. The scenarios' ban on an external controller excludes such additional automation, not
their stated scripted mouse/keyboard inputs, map actions, the original's computer player or a
scenario's stated direct order given by the harness, as in M-0023 (B49). Any game speed, held
fixed within a run: no scenario here observes a delay that the game speed changes. A rule that
waits on a speed-normalised delay (glossary, claim G3) inherits that dependence, for example
claim 11's infantry readiness while a *cheer* plays; where an infantry unit of a scenario may
play such a sequence, its runs are not compared across game speeds. The recorder reads, on every
tick, each actor's current, queued and suspended behaviour, the tick on which its processing
timer next runs out, its target and its destination, and, where a scenario says so, its queued
destinations, its house's skill level, its contact, and whether its distributed-fire candidate
list is empty; "processing tick" means a tick on which that expiry tick moves forward. Scenarios
with units of a computer-controlled house test the original's own rules, including the parts
marked as the original's only (notes for the AI designers). A scenario that has a test map place
an object with a behaviour expects that behaviour to be current once the object is placed; a run
in which it is not is invalid (repeat it), not a falsification. A scenario that looks for draws
has the recorder also read the position of the scenario random stream in its stream sequence
(S-0202) at the start and end of every tick, which gives the number of raw values taken in the
tick. A draw *U*(*a*, *b*) with *a* < *b* takes at least one raw value (S-0202, claim 4), so a
processing that takes one advances the stream further. The scenario compares ticks on which the
actor under test processes with otherwise similar ticks on which it does not, so that raw values
taken by other rules (movement, other actors) cancel out on average; "an extra draw" means that
processing ticks take more raw values on average than the comparison ticks.

*Houses.* Every scenario is a skirmish with one human player, the *human-controlled house*, and
one enemy bystander computer player (glossary). Scenario B28 has one allied and one enemy
bystander computer player, made allied with the human player or not by the lobby teams. In this
section "a computer-controlled house" and "an enemy house" mean a bystander: the enemy one unless
the scenario says allied. A scenario's *test objects* are every object it names or places, of
any house: the objects under test, their targets, attackers and helpers. Test objects owned by a
bystander are its *named test objects*. A scenario may add named test data meant to hold a
bystander back, but its void rules still apply.

*Observed areas.* Each bystander's start position is the one farthest from the test objects. A
scenario's observed area is every cell within 10 cells of any position that one of its test
objects has during the run (so routes and flight paths are included), with the exceptions given
under B11 (f), B20 and B28.

*Void rules.* Two rules apply to every scenario, besides any invalid run the scenario names:
- (a) The glossary's void rule: a run is void, and is repeated, when an object of a bystander that
  the scenario does not name enters the observed area or acts on a test object, or when a
  bystander is defeated before the observation ends.
- (b) For named test objects of a bystander: the recorder also logs each order that such an object
  receives and who gives it, and the object's team membership. A run is void, and is repeated,
  when such an object gets an order or a team-script instruction from its computer player that
  the scenario does not set up (through its procedure or a test-map team it names), or joins a
  team that the scenario does not name. Rule (b) looks at the order or the team, never at the
  behaviour that follows, so a falsifying result is never set aside by it. It does not apply to
  the parts whose subject is the computer player's own decisions or a test-map team script: B4's
  team-member runs, B9 (d), B10 (e), B11 (c), B11 (f), B20, B21, B25 (b) and B26. Rule (a) still
  applies to them, except B20, which fixes a void rule of its own (glossary).

Where a scenario moves or orders a named test object of a bystander (an enemy driven out of
range, walking into range or luring a unit away), it does so through test-map data, a map
trigger or a test-map team whose script gives that order, and such an order counts as set up for
rule (b). A team made for this purpose changes the team-member rows of claims 14 and 15 for that
unit only, which matters only where that unit is itself the subject.

*On our engine.* When a scenario is replayed on our engine, each bystander's counterpart is an AI
house that gives no orders (glossary). The rules that a scenario tests for computer-controlled
owners, the parts marked as the original's only, have no counterpart for player houses in our
engine: there the named test objects of that AI house follow the rules given for
human-controlled owners (Terms, houses in our engine), so those parts are not replayed.

- **B1 (claims 1, 2, 4).** Create one infantry unit, one vehicle and one aircraft (a type that can
  stand landed on open ground) of a human-controlled owner, and the same of the
  bystander (named test objects). Put each into *sleep*,
  *harmless*, *stop*, *ambush*, *wait* and *return* in turn (through a test map that places
  objects with these behaviours) and record 20 processing intervals each. Expected: every interval
  is exactly 450 ticks; a newly created actor shows *none* in all three slots before anything sets
  them. Falsified by any interval other than 450 while the behaviour stays current, by a
  draw-dependent spread, or by a newly created actor showing a behaviour other than *none* in any
  slot before anything sets them. A run in which another rule (for example an aircraft rule) changes the
  behaviour before 20 intervals have been recorded keeps the intervals recorded until then. Runs:
  one per kind and behaviour.
  Building *move* is tested by S-0402 claim 27a and scenario H25's site-offer variants:
  record processing ticks and random-stream state while the field-seeking building's
  accepted *move* is current. Expected: no movement, packing or guard-spot clearing,
  450 ticks between processings, no processing draw, and no cancellation of *move* when
  the group returns to rest. H25 specifies the setup, repeats and falsifying results,
  and separates group-update draws from behaviour-processing draws.
- **B2 (claim 6, F1, F2).** Named test file: rules that set the rate of *move* to 0.06 for one run,
  0.05 for another and `6%` (written with a percent sign) for a third. Order a vehicle back and forth between two cells 40 cells apart and
  record its processing intervals while it moves, counting only intervals between two *move*
  processings of the same move. Expected: intervals of 53, 54 or 55 for 0.06 and of 45, 46 or 47
  for 0.05, each value seen at roughly a third of the time over 300 intervals. Falsified by any interval
  outside the expected set of its rate, in particular any 56 with 0.06, or by no 53 at all in 300
  intervals with 0.06 (which would mean decimal rather than single-precision arithmetic). For
  `6%`, a double product rounded toward zero (claim 6, F1): expected 53, 54 or 55 as for 0.06,
  falsified by any 56 or by no 53 in 300 intervals. Runs: as many moves as needed for 300 intervals
  per rate.
- **B3 (claims 3, 7 to 9, 11, 12, 22).** (a) Order a stationary *guard* vehicle to move; expected:
  *move* becomes current at the vehicle's first update after the order takes effect (S-0203,
  claim 7) and is processed on that tick.
  (b) While it moves, order it to attack a distant stationary enemy building out of range; expected:
  *attack* stays queued while the vehicle moves under *move* and becomes current only on a tick on
  which it is not moving. (c) Repeat (b) with an infantry unit; expected: the same. (d) Give a moving
  vehicle a second move order to another cell; expected: no slot changes, the destination changes.
  (e) Let a vehicle arrive; expected: on the first due processing with no destination,
  no movement and nothing queued, its resting behaviour is queued. Record the slots and
  processing timer at processing exit and update end. If otherwise ready at the late start,
  rest is current at update end, without a second processing that turn; its first processing
  follows on the next update. Otherwise it waits for the next permitted start (claim 11). (f) Force: a vehicle of a computer-controlled
  owner on a long move is taken over by a mind-control unit of the human-controlled house. Expected:
  on the tick of the takeover *guard* is current, nothing is queued and target and destination are empty, although the vehicle
  was moving (not ready to change). (g) Stop order: give a moving vehicle in *move* the stop order.
  Expected: no slot changes on that tick, destination empty; at a processing after it has stopped
  moving, its resting behaviour is queued. Repeat with a harvester in *harvest*; expected: *guard*
  current on that tick. Repeat with a slave master whose harvesting slaves are out working,
  expected: on that tick every slave has *guard* current and an empty target and destination. (h) Factory exit cell: a test map places an armed vehicle, in contact with
  nothing, on the cell (*x*, *y* + 1) of a vehicle factory whose foundation's top-left cell is
  (*x*, *y*), and a second one on a cell next to the factory; give both an attack order on an enemy
  in range. Expected: the second vehicle's *attack* becomes current on that tick, the first one's
  stays queued while it stands there. (i) Refusals for *wait*: a test map places an armed infantry
  unit of the human-controlled house in *wait*. Give it a guard order; expected: nothing queued,
  *wait* stays current. Then give it a move order; expected: *move* queued and current at the next
  per-tick start at which it is ready. A second unit placed in *wait* by the same map, of a
  computer-controlled house, is taken over by a mind-control unit of the human-controlled house; expected:
  target and destination empty, but *wait* still current (the forced *guard* is refused).
  (j) Refusal for *selling*: an armed defensive building of the human-controlled house is sold; while
  *selling* is current, give it a force-fire order on an enemy in range, if the input allows
  one; expected: nothing queued and *selling* stays current (if the input gives no order to a
  selling building, the case is recorded as not applicable). (k) Timer reset by a start: named
  test file setting the rate of *guard* to 0.2 (*D*<sub>r</sub> = 180). An armed vehicle of the
  human-controlled house rests in *guard* with no enemy within sight (processing intervals 180,
  181 or 182). Ten ticks after one of its *guard* processings, order it to move 10 cells.
  Expected: *move* becomes current at the vehicle's first update after the order takes effect and
  is processed on that tick, about 170 ticks before the *guard* timer would have run out. (l)
  Queue replacement: give a vehicle of the human-controlled house in *move* (moving, so not ready
  to change) an attack order on a distant enemy building out of range, then, while it still
  moves, a move order to another cell. Expected: after the attack order the queued slot shows
  *attack*; after the move order it shows *move* (the current behaviour, queued again in place of
  *attack*); *move* stays current for as long as the vehicle is still moving (on the tick of its arrival, the
  late start of claim 11 may already make the queued behaviour current; that tick is not part of
  "throughout"). Falsified by *move* not becoming
  current at the vehicle's first per-tick start after the order takes effect, or not being
  processed on the tick it becomes current (case a), by a queued behaviour
  becoming current while the actor moves (cases b, c), by no resting behaviour queued at the first
  qualifying processing after the vehicle stops, by rest still queued at update end when
  ready at the late start, or by a second processing in that same turn (case e), by any slot change in (d) or (g) (except the harvester's *guard*, the slaves'
  *guard*, and the *guard* of a slave master whose own type is a harvester in *harvest*),
  by a slave without *guard* current, or with a target or destination, on the tick of the stop
  order in (g), by *guard*
  not current at once in (f), by the first vehicle of (h) starting *attack* on its cell, by *guard*
  queued or current in (i), by the move order not being queued in (i), by *attack* queued in
  (j), by a first *move* processing in (k) only when the *guard* timer runs out, or in (l) by
  *attack* still queued after the second order, or by any change of the current behaviour. Runs:
  three each.
- **B4 (claims 14, 15).** Named test file: AI settings with a maximum skill level of 5 and an
  area-guard threshold of 0 for run T0, of 1 for run T1 and of 6 (above the maximum) for run T6,
  so that the result does not depend on the stock values. Skirmish, so the human-controlled house
  has skill level 0 and the computer-controlled house has 5 (claim 15). In each run, compare the
  resting behaviours after a completed move, in units not in a team. Human-controlled house:
  - an armed infantry unit: expected *guard* in every run,
  - an armed vehicle: expected *area guard* in T0, *guard* in T1 and T6,
  - the same infantry unit and vehicle with a second test file that sets the default-to-area-guard
    flag on their types: expected *area guard* in every run,
  - a veteran or elite unit whose ability list contains the guard-area ability: expected *area
    guard* in every run,
  - an unarmed vehicle: expected *guard* in every run,
  - placement fall-back: an ore harvester built by a factory that stands on a cell without ore:
    expected *harvest* queued on placement; the same harvester ordered to move to a cell without
    ore and arriving there: expected *guard*.

  Computer-controlled house (the original-only rows, notes for the AI designers): the test map
  places an armed infantry unit, an engineer, a vehicle thief and an armed vehicle in *move* with
  no destination, so that their first *move* processing falls back to rest (claim 22) without any
  team or trigger: expected *area guard* in T0 and T1, *guard* in T6 (claim 14 rows 7 and 9,
  claim 15 rows 10 and 12). The same map also places an unarmed infantry unit that is neither an
  engineer nor a vehicle thief: expected *guard* in every run (claim 14 rows 7 and 10). A second
  armed infantry unit and armed vehicle of that house, members of a test-map team whose script
  only waits, placed the same way: expected *guard* in every run (claim 14 row 12, claim 15 row
  11).
  Falsified by any other resting behaviour, or by a recorded skill level other than 0 for the
  human-controlled house or 5 for the computer-controlled house (claim 15 gives those levels in a
  skirmish). Runs: one per case and run.
- **B5 (claim 16).** Named test file: rules giving *harmless* the no-threat property and neither the
  zombie nor the retaliate property. Test map: a unit of one house placed in *harmless* next to an
  enemy unit in *guard*. Expected: the guarding unit never targets it by itself over 2000 ticks.
  Then damage the harmless unit with a third unit; expected: on that tick it falls back to rest, and
  the guarding unit targets it within one targeting delay. Falsified by a target choice before the
  damage, or no fall-back after it. Runs: three. (b) Zombie suppresses the fall-back: the same map
  with a second test file that also gives *harmless* the zombie property. Expected: the damaged unit
  does not fall back to rest, *harmless* stays current, and the guarding unit still never targets it
  by itself over 2000 ticks. Falsified by a fall-back after the damage. Runs: three. (c) Retaliation
  first: a third test file giving *harmless* the no-threat and the retaliate properties and not the
  zombie property, giving *attack* neither the no-threat nor the zombie property, and turning player
  return fire on (claim 18). The subject is an armed vehicle of the human-controlled house whose
  weapon can fire at vehicles, placed in *harmless* with no target and no destination. An enemy
  vehicle, within the subject's weapon range and ordered by a test-map team to attack it, hits it.
  Record the subject's target each tick, and its current, suspended and queued behaviours at the end
  of the tick of the first hit. Expected: *attack* current, *harmless* suspended and nothing queued.
  Falsified by any other behaviour current, suspended or queued at that end, such as *guard* or
  *area guard* queued or current (the fall-back first: claim 15, rows 10 to 12). A run in which the
  subject has a target before the first hit, or in which its target at that end is not the attacker,
  is void. Runs: one. (d) Healing hit: the first test file. An infantry subject of the
  human-controlled house is placed in *harmless* with reduced health, next to an own-house healer
  that is ordered to heal it, with no enemy within 10 cells. Record the subject's health and its
  behaviour slots each tick until it is at full health. Expected: its health rises, *harmless* stays
  current and nothing is queued. Falsified by a fall-back to rest, or anything queued, on a tick on
  which its health rises. A run in which its health never rises is void. Runs: one.
- **B6 (claims 17, 19, 25).** Named test file: rules giving *sticky* the paralysed and the scatter
  properties and turning its retaliate property off, giving *sleep* the zombie property, the
  scatter property and no retaliate property, giving *guard* the paralysed and the scatter
  properties with no retaliate property, giving *area guard* the zombie and the scatter properties
  with no retaliate property, and turning the player-scatter setting on (case 2 of claim 19 then
  holds for a human-controlled owner, as it does for every house in our engine). Test map places
  unarmed vehicles of the human-controlled house of an ordinary driving stock type: *K* in
  *sticky*, *Z* in *sleep*, *G* in *guard* and *A* in *area guard*, each on open ground with no
  unit within 2 cells. (a) Give *K* and *Z* a move order. Expected: *K* never starts *move* (the
  move stays queued); *Z*, if ready, starts *move*. (b) Damage *G*, *A*, *K* and *Z* from out of
  their range while they stand still with no destination. Expected: *G* and *A* move away to
  scatter (the scatter from damage names no point, so the paralysed refusal of claim 17 does not
  apply, and S-0207 claim 16 lets them accept). *K* and *Z* do not move: a vehicle in *sticky* or
  *sleep* refuses every scatter request (S-0207 claim 16, case a). Repeat with the scatter
  property off for *G* and *A*: expected no scatter (case 2 of claim 19 needs the property).
  Original-only variant (claim 19, case 2 for a computer-controlled owner): repeat the damage of
  *G* and *A* with the player-scatter setting off (a second test file that differs only in that
  setting), once with *G* and *A* owned by the bystander (named test objects) and once with them
  owned by the human-controlled house. Expected: the computer-controlled *G* and *A* scatter, the
  human-controlled ones do not. (c) *G* stands on a cell of a bunker of its own house that has
  room for one vehicle and no other unit on its cells; another vehicle of the same house is
  ordered into the bunker. Expected: once the entering vehicle has stopped on the bunker's cell,
  *G* stays where it is. Control run with a test file that turns the paralysed property of *guard*
  off: expected *G* moves away from the bunker. Factory variant: use S-0417 U5's blocked-apron
  setup, with *G* on one apron neighbour and a stationary blocker on the apron itself. Observe the
  first clearing request while the blocker remains. Expected: *G* refuses the neighbour's
  threat-point request with the paralysed property on; with it off, *G* accepts and scatters under
  S-0207 claim 16. Record the request options, destination, movement and scenario-stream state.
  One run per setting; movement with the property on falsifies the refusal. A run without a
  blocked-apron clearing request is void. Falsified by *K* starting *move* in (a), by *Z* not
  starting *move* at a per-tick start at which it is ready, by a scatter of *G* or *A* with the
  property off in (b), by *G* or *A* not scattering with the property on in (b), by *K* or *Z*
  scattering in (b), in the variant of (b) by a computer-controlled *G* or *A* that does not
  scatter or a human-controlled one that does, or in (c) by *G* moving away with the paralysed
  property on. A (c) control run in which *G* does not move either leaves claim 17's refusal
  untested. (d) Case 1 of claim 19: named test file turning the player-scatter setting on. An
  armed vehicle of the human-controlled house stands in *guard* with no target and no destination.
  An enemy unit whose weapon outranges the vehicle's fires at it from beyond the vehicle's weapon
  range and beyond (*S* + 0.5) × 256 leptons, *S* its sight (F7), so it does not retaliate.
  Expected: the vehicle moves away to scatter, with no override to *attack* (the suspended slot
  stays empty). Control 1: the player-scatter setting off and no scatter veterancy ability:
  expected no move. Control 2: the attacker within (*S* + 0.5) × 256 leptons but beyond the weapon
  range: expected an override to *attack* and no scatter move. Falsified by no move in the main
  run, by a move in control 1, or by a scatter move instead of the override in control 2. Runs:
  three each. (e) Fall-back rows (claim 17; claim 14 row 6, claim 15 rows 7 and 9): a named test
  file gives *move* the zombie property (run Z), a second one gives it the paralysed property (run
  P), and the stock rules serve as control (run C). On a test map with no object of another house,
  the human-controlled house has an armed infantry unit of a type that does not default to area
  guard and has no guard-area veterancy ability, an armed vehicle that is not a harvester, and an
  unarmed vehicle that is not a harvester, a loaner or a deployer. Each is ordered to move 5 cells
  and arrives. Expected in Z and in P: at the first *move* processing after it has stopped, the
  infantry unit and the armed vehicle queue nothing (the infantry unit also at the end of each
  update, by claim 22's end-of-update check), *move* stays current and the next processing comes 1
  tick later, and so on over 20 processings (each one falls back to rest again and reaches row 6
  or 9); the unarmed vehicle queues *guard* at that processing (row 7 comes before row 9), which
  becomes current at the next permitted start, including that update's late start when ready.
  Expected in C: the two vehicles queue a resting behaviour at that processing (*area guard* or
  *guard* for the armed vehicle by claim 15 rows 10 and 12, *guard* for the unarmed vehicle), and
  the infantry unit has *move* current and *guard* queued at the end of the first tick at whose
  end it has no destination (claim 22's end-of-update check, as in B49 (b)), before any *move*
  processing after it has stopped. A run in which the recorder shows a target, a destination or
  queued destinations at that processing, or for the infantry unit in C at the end of that tick,
  is invalid. Falsified in Z or P by a behaviour queued for the infantry unit or the armed
  vehicle, by *move* not staying current, or by no *guard* queued for the unarmed vehicle; in C by
  nothing queued for a vehicle at that processing, or by *guard* not queued for the infantry unit
  at the end of that tick. Runs: three per run and unit.
- **B7 (claims 8, 10, 12, 18).** (a) A human-controlled armed vehicle in *move* (long route) is fired at by
  an enemy within its weapon range; expected: no retaliation, *move* continues. (b) The same vehicle
  in *guard*, fired at by an enemy land vehicle that neither sinks nor crashes; expected: an
  override to *attack* (the suspended slot shows *guard*), and a revert to *guard* on the tick of
  the killing hit on the attacker; on the tick of the override the expiry tick of its processing
  timer does not change (claim 3), so the first *attack* processing comes when the timer of its
  last *guard* processing runs out. (c) With a test file turning player return fire on, repeat (a).
  Expected: retaliation. (d) An attacker with longer range at a distance just above and just below
  (sight + 0.5) × 256 leptons: override only below. (e) Override while a behaviour is queued: named
  test file giving *sticky* the retaliate property and turning player return fire on (claim 18); a
  test map places an armed vehicle of the human-controlled house in *sticky*, a move order queues
  *move* for it (it stays queued, claim 25), and an enemy in range fires at it. Expected: on the tick of the override the suspended slot
  shows *move*, not *sticky* (the queued *move* may then start at the next per-tick start, since
  the vehicle is no longer in *sticky*, claim 11), and after the attacker is destroyed the vehicle is in
  *move* and never returns to *sticky*. With the quirk off, the suspended slot would show *sticky*
  and the revert would bring *sticky* back. (f)
  Refused override: named test file giving *selling* the retaliate property; an armed defensive
  building of the human-controlled house, with no target, is sold while an enemy in range fires
  at it. Expected:
  while *selling* is current, its target becomes the attacker and its slots do not change.
  Falsified by retaliation in (a); in (b), in a run in which the vehicle had no target when it
  was first hit, by no override to *attack* with *guard* in the suspended slot, by no revert to
  *guard* on the tick of the killing hit on the attacker, or by a change of the expiry tick on the tick of the override; by no retaliation in (c); by a wrong distance boundary in (d), by suspended *sticky* or a
  return to *sticky* in (e), or by an unchanged target or a slot change in (f). (g) Existing target:
    a human-controlled armed vehicle in *guard* has taken an opportunity target A (an enemy in its
    weapon range; the recorder shows the target). A second enemy B, beyond the vehicle's weapon range
    and inside its sight, then fires at it. Expected: no override, the target stays A and the
    suspended slot stays empty. Repeat with a vehicle of the computer-controlled house (the
    original-only part of claim 18), choosing A and B so B's threat sum is at least A's
    (S-0309 claim 25 part h): expected an override to *attack* with B as target at once.
    A third run with the computer-controlled vehicle, with B beyond (*S* + 0.5) × 256 leptons and
    beyond its weapon range, but still with B's threat sum at least A's: expected an override
    at once as well (the human distance limit does not apply). Repeat the second run with B's
    threat sum below A's: expected no retaliation, A retained. Include equality in the
    at-least case. The recorder reads both threat sums at the hit; reject a run in which their
    intended relation or another retaliation condition fails. Falsified in the first run by an
    override to *attack* or by B becoming the target, in the second and third by no override,
    or in the lower-threat repeat by B becoming the target.
    (h) Order during an override: repeat (b) and, while the override is in effect, give the
    vehicle a move order (which clears its target, S-0213). Expected: *guard* never becomes
    current by a revert; *move* becomes current through the queue (claim 12), and the suspended
    slot still shows *guard* after that. Falsified by an empty suspended slot after the order (a
    revert empties it).
    (i) Killing hit on an attacker that stays in play: repeat (b) with an enemy infantry attacker
    that plays a death sequence when the vehicle kills it (S-0312). The recorder also reads the
    attacker's health and the tick on which the attacker is removed. Expected: the revert
    (*guard* current, suspended slot empty) on the tick on which the attacker's health reaches 0,
    while the attacker is still in play. Falsified by no revert on that tick, or by a revert on
    the tick of the attacker's removal or later.
    (j) Attacker going off the map: a test map places an armed vehicle of the computer-controlled
    house in *guard* (in the original it overrides at once, claim 18), and next to it an infantry
    unit and an empty transport of the human-controlled house. Scripted input orders the infantry
    unit to attack the vehicle and, after its first hit, to enter the transport. The recorder also
    reads whether the infantry unit is on the map. Place the transport touching the infantry
    unit, give the enter order on the tick of the first hit, and use an infantry type that the
    vehicle's weapon needs more than one hit to kill. Expected: an override to *attack* with the
    infantry unit as target, and the revert (*guard* current, suspended slot empty) on the tick
    on which the infantry unit goes off the map. If the vehicle already had the infantry unit as
    target before the hit, the recorder shows that unit as the vehicle's target again after the
    revert, and otherwise no target; this is expected and does not falsify the case. Falsified by
    the override still in effect on a later tick while the infantry unit is inside the transport,
    or by a target after the revert other than the one the vehicle had before the hit. A run in
    which the infantry unit is destroyed before it boards is invalid; make up to ten attempts to
    get three valid runs.
    (k) Owner change of the attacker: repeat (b) with an enemy infantry attacker, and with an
    infantry unit of the human-controlled house that has a mind-control weapon placed within
    reach of the attacker. Once the override is in effect, scripted input orders that unit to take
    the attacker over. The recorder also reads the attacker's owner. Expected: the revert
    (*guard* current, suspended slot empty) on the tick on which the attacker's owner changes.
    Falsified by the override still in effect on a later tick while the attacker belongs to the
    human-controlled house. A run in which the attacker is destroyed before the takeover is
    invalid.
    (l) Later revert by another target: repeat (h); the attacker of (b) may go on firing, since a
    human-controlled vehicle in *move* or *attack* does not retaliate (claim 18), so its fire
    changes no slot. A run in which the vehicle is destroyed before the killing hit on C is
    invalid; make up to ten attempts to get three valid runs. A test map also places an unarmed enemy vehicle C that the
    vehicle's weapon needs several hits to kill. Once *move* is current (the suspended slot still
    shows *guard*, as in (h)), scripted input orders the vehicle to attack C. The recorder also
    reads C's health and the vehicle's target and destination. Expected: *attack* becomes current
    through the queue with *guard* still suspended; on the tick on which C's health reaches 0,
    *guard* is current, the suspended slot is empty and the vehicle has no target and no
    destination. Falsified by an empty suspended slot at any tick between the move order and the
    killing hit on C (with the switch off it is empty from the start-next that makes *move*
    current), by *guard* still suspended after the tick of the killing hit, or by *attack* still
    current on that tick (with no revert, *guard* would come only at a later *attack* processing,
    claim 23). A run in which the vehicle's own processing of *attack* falls on the tick of the
    killing hit is still decided by the suspended slot, which only a revert empties.
    Runs: three per case.
- **B8 (claims 21, 32).** Named test file: rules that give *guard* an anti-air rate different from
  its rate (for example rate 0.07, anti-air rate 0.2). Record the processing intervals of an armed
  building without targets. Expected: 180, 181 or 182; an armed vehicle in *guard* at the same time
  shows 63, 64 or 65 (when not rearming; 0.07 is held as 0.0700000002980232, so
  *D*<sub>r</sub> = 63). Falsified by the building using the rate or the vehicle the anti-air rate.
  Runs: 200 intervals.
- **B9 (claim 23, F3, F4).** Named test file: rate of *attack* 0.07 (*D*<sub>r</sub> = 63). An
  infantry unit with a weapon of range 512 leptons or less attacks a stationary target that cannot
  move; record intervals while the 2-D distance is 300 to 700 leptons, and while it is beyond 768
  (approaching). Expected: 31 or 32 at close range (trunc(63/2) = 31, trunc(64/2) = trunc(65/2) =
  32, so 32 about twice as often as 31) and 63, 64 or 65 beyond. Falsified by no halving, or
  halving outside the band. Runs: repeated approaches until 100 intervals per band are recorded.
  (b) Explosive-charges row: no runnable scenario is established. An attack order is translated
  into *sabotage* by S-0214, and the former retaliation proposal is excluded by S-0309 claim 18.
  This row is unmeasured, not supported by discarding every non-retaliating run.
  (c) Deployed infantry of the human-controlled house: a deployer type
  that deploys to fire, deployed, ordered to attack an enemy in weapon range, with the rate of
  *attack* set to 0.07 and the close-range flag set on its type by the test file, so that the
  halving of the general rule would apply in the close-range band. Expected: intervals 63, 64 or 65 even when the 2-D
  distance is in the close-range band (no halving). Falsified by intervals of 31 or 32, or by
  intervals of 1 tick. (d) Computer capture row (the original's only): a test-map team whose script
    tells a computer-controlled infantry unit whose type infiltrates to attack an enemy building it
    can reach. Expected: at its first *attack* processing with that building as target, *capture* is
    current on the same tick with the building as destination, and the next processing comes exactly
    1 tick later with no extra draw on the processing tick. Falsified by *capture* only queued, by
    another interval, or by an extra draw. Runs: 100 intervals in (c); three in (d).
- **B10 (claims 24, 26, 33, F5, F6).** (a) An armed vehicle in *guard*: record targeting-timer
  restarts; expected intervals *N* + 0, 1 or 2 with the normal targeting delay. (b) The same in
  *area guard*; expected the area-guard targeting delay + 0, 1 or 2, and processing intervals
  *D*<sub>r</sub> + 1, … or 5. (c) Let an enemy lure the *area guard* vehicle away: expected the
  vehicle drops its target and turns back on the first processing at which its 3-D distance to the
  guard spot exceeds *L* of F5 (with a test file giving its type a guard range of 4 cells, so
  *G* = 2048 and *L* = 2252 leptons, not 2048; falsified by a turn-back near 2048). In the same
  processing, if its targeting timer has run out, a target search around the guard spot is
  recorded. (d) A vehicle in *guard* that is rearming: processing intervals equal to the remaining
  rearm time. (e) Drop list: a test-map team whose script assigns *sleep* to its member, an armed
  vehicle in *guard* that has just taken an opportunity target by its own per-tick search. Expected:
  on the tick *sleep* becomes current, the target is empty. Control: the same with a script that
  assigns *hunt* (not in the drop list); expected: the target is kept on the tick *hunt* becomes
  current. (f) Area guard, harvester row: an ore harvester of the human-controlled house placed in *area guard*
  by a test map, ready to change. Expected: at its first processing *harvest* becomes current on
  that tick, and the next processing comes 2 to 11 ticks later (1 + *U*(1, 10)), each value seen
  over 100 placements. (g) Area guard, guard-spot rule: an armed vehicle of the human-controlled
  house placed in *area guard* with no guard spot and nothing queued. Expected: after its first
  processing its guard spot is its own cell. Falsified by other intervals, another leash distance,
  no target search around the guard spot in a turn-back processing at which the targeting timer
  had run out (c; the vehicle is armed and in *area guard*, so the target-acquisition specs allow
  it a passive search; a run in which they would not is invalid),
  a kept target in (e), a dropped target in the control of (e), a delay outside 2 to 11 or a
  missing *harvest* in (f), or no guard spot in (g). Runs: 100 intervals per case, 5 lures, three
  runs of case e and of its control, 100 placements in (f), three in (g). (h) Close-range
  shortening of *area guard* (F3): named test file giving *area guard* the rate 0.07
  (*D*<sub>r</sub> = 63) and an armed vehicle type a primary weapon range of 2 cells (512 leptons).
  The vehicle of the human-controlled house stands in *area guard* on its guard spot; a stationary
  enemy building is placed within its weapon range and at a 2-D distance of 300 to 500 leptons.
  Expected: intervals 10 or 11 (trunc(64 / 6) = trunc(65 / 6) = 10, trunc(66 / 6) = trunc(67 / 6) =
  trunc(68 / 6) = 11), both values seen. Control: the building at a 2-D distance above 768 leptons
  and inside the leash: expected 64 to 68 (trunc(*D* / 6) not applied). Falsified by an interval
  outside 10 to 11 in the main run, or by a shortened interval in the control. Runs: 100 intervals
  each. (i) Aircraft factor: the same test file; an armed aircraft of a type that can stand landed,
  of the human-controlled house, placed in *area guard* by a test map with no target. Expected:
  intervals 127 to 131 (2 × 63 + *U*(1, 5)), each value seen; a vehicle in *area guard* at the
  same time shows 64 to 68. Falsified by intervals of 64 to 68 for the aircraft. Runs: 100
  intervals. (j) Bunker row: an armed vehicle of the human-controlled house stands in a bunker of
  its own house (ordered in); give it an area-guard order. Expected: *area guard* becomes current,
  and at its first processing *guard* becomes current on that tick, the next processing coming
  exactly 1 tick later (and the guard spot not taken as its own cell, claim 26). Falsified by
  *area guard* staying current, or by another interval. Runs: three. (k) Search that finds a
  target (claim 26, T2): named test file giving *area guard* the rate 0.07 (*D*<sub>r</sub> = 63).
  The same file turns the no-threat property of *sleep* off, so that the search may pick an actor
  in *sleep* (claim 16). A test map places an armed vehicle of the human-controlled house in *area
  guard* on its own cell, and an enemy vehicle of the bystander (a named test object), in *sleep* (which does nothing by itself, claim 4), within the first
  vehicle's weapon range.
  Expected: at the processing whose search first finds the enemy (the recorder shows the target
  from that tick on), the next processing comes exactly 1 tick later, not 64 to 68 ticks (nor 10
  or 11 under the close-range condition of F3). A run in which the target first appears on a tick
  on which the vehicle does not process, or never appears, is invalid. Falsified by any other interval after that
  processing. Runs: 20.
- **B11 (claim 27).** (a) Skirmish: a computer-controlled unit placed in *hunt* on an island it cannot
  leave, with no enemy reachable or in range. Expected: it stays in *hunt* without moving; with a
  test file setting the stupid-hunt flag on its type, also no target is taken when an enemy comes
  into range. Falsified by the unit moving towards the enemy base, or by target searches with the
  flag. Runs: three. (b) Infantry capture rule: a computer-controlled infantry unit whose type has
  the infiltrate flag, placed in *hunt* near an enemy building it can reach. Expected: at the first
  *hunt* processing at which its target is that building, *capture* is current on the same tick,
  with the building as destination, and the first *capture* processing comes exactly 1 tick
  later; that *hunt* processing restarts no targeting timer and shows no extra draw (common
  setup). A run in which its target never becomes that building is invalid. Falsified by
  *capture* only queued, by an interval other than 1 tick, or by a targeting-timer restart or an
  extra draw at that processing. (c) Infantry undeploy rule: a
  test-map team whose script assigns *hunt* to a computer-controlled deployer infantry unit that
  is deployed and has no target. Expected: at its first *hunt* processing it starts its *undeploy*
  sequence, and its next processing comes exactly 1 tick later. Falsified by no *undeploy*, or by
  another interval. (d) Vehicle deploy rule: named test file setting the rate of *hunt* to 0.07
  (*D*<sub>r</sub> = 63). A test map places a vehicle of the human-controlled house whose type
  deploys into a building, in *hunt*, on open ground with room for that building. Expected: *hunt*
  stays current, with nothing queued, from its first processing until the vehicle is replaced by
  the building, and every processing interval in that time is 63, 64 or 65. Falsified by another
  behaviour current or queued before the building appears, or by other intervals. Runs: three in
  (b), (c) and (d). (e) General with-target outcome (claim 6, claim 27): named test file giving
  *hunt* the rate 0.07 (*D*<sub>r</sub> = 63), *capture* 0.05 (45) and *sabotage* 0.2 (180). A
  test map places an engineer of the human-controlled house in *hunt* near an enemy building it
  can capture, and, in other runs, an infantry unit with explosive charges near an enemy building
  that accepts them. Expected: at the first *hunt* processing at which the unit's target is the
  building, *capture* (engineer) or *sabotage* (charges) is current on that tick with the building
  as destination, and the next processing comes 45, 46 or 47 ticks later (*capture*) or 180, 181
  or 182 (*sabotage*), not 63 to 65. A run in which the new behaviour is only queued (the unit
  was not ready to change) is set aside; in it the interval is 63 to 65. Falsified by an interval
  of 63 to 65 in a run that is not set aside. Runs: 20 per unit kind, with each of the three
  values seen at least once.
  (f) Per-tick *hunt* of base-building vehicles (the original's only): a skirmish on a test map
  with no pre-placed objects and the fewest starting units, so that each house starts with only
  its base-building vehicle (a vehicle whose type deploys into a construction-yard type) and no
  building. The recorder also reads each house's automatic base-building setting and its number of
  construction yards. Expected: on the first tick on which the computer-controlled house's setting
  is on, its vehicle (whose current behaviour is then neither *hunt* nor *unload*) ends the tick
  with *hunt* current, or with *hunt* queued if it is not ready to change, and then deploys
  through the vehicle rule of claim 27. The human-controlled house's vehicle, left without orders,
  never has *hunt* queued. A run in which the computer-controlled vehicle has *hunt* or *unload*
  current before the setting turns on, or the setting never turns on (it is the computer player's,
  Terms), is invalid. Observed area: the cells within 10 cells of any position of the
  human-controlled house's vehicle, and the bystander's base-building vehicle is a named test
  object; the observation ends when that vehicle has deployed. Falsified by a valid run in which
  that tick ends with *hunt* neither current nor queued, or ends with *hunt* queued while the vehicle is ready to change, or by *hunt* queued
  for the human-controlled vehicle. Runs: three.
- **B39 (claim 29, depot processing and interruption).** Use S-0314 H16's land-depot
  setup and S-0504 N8's naval-yard setup, retaining their game speed, houses, observed areas
  and void rules. Stock data only, no mods or external automated player. In three runs per
  setup record processing ticks, reply categories, current/queued behaviours, contact,
  tether, destination, body facing, motion, health and credits with a read-only recorder.
  First let an entry complete without further input. Expected: contact precedes arrival.
  movement arrival in the supplied docking-point cell queues repair/sleep without requiring
  exact horizontal alignment or a pre-existing tether, and without bypassing readiness.
  Arrival itself requests no map-east turn, including when tethered, and schedules none
  for a later enter processing. If otherwise ready, sleep starts at that update's late
  start. An existing driving turn can hold it queued under claim 11; ship or hover body
  rotation alone cannot. Reuse S-0314 H16's three-locomotion-kind and existing-turn
  controls, including its inconclusive rule for states the fixture cannot reach. A new
  turn requested solely by movement arrival falsifies the claim. Record any separate
  completed-entry confirmation independently; no reservation-only paid step.
  Reuse S-0314 H19's tether and off-centre
  controls and H22's late-retry controls, with their expected results and falsifiers:
  same-turn destination assignment does not repeat movement or due behaviour processing,
  and competing retries use object-turn order. An early unarmed guard transition must meet
  the truncated centre-distance and stillness tests and choose a 1-tick processing delay.
  A paid step at reservation, a forced queue start, or that guard result at 64 leptons or
  farther falsifies the respective claim. At each repeatable pre-tether and post-tether
  point in separate runs issue move, attack, stop, or a repeated enter to the same building.
  Compare contact and destination changes at order application with S-0213 claims 5 and 8
  and S-0314 claims 20 to 23, rather than predicting that every order breaks a tether.
  A same-building repeated enter while already in *enter* changing contact or destination,
  or an untethered non-accepting entry reply retaining contact instead of falling back,
  falsifies claim 29. Test docked *sleep* separately: its same-depot enter is not ignored.
  Reuse S-0314 H18 for retained docked contact, per-tick queued-entry retries and sale's
  run-away notice, with that scenario's expected results and falsifiers. If the
  fixture cannot isolate early guard from completed arrival, that sub-test is inconclusive.

- **B12 (claims 28 to 31; part e for the scatter off a building).** (a) A computer-controlled
  engineer placed in *capture* without a target: expected *hunt* queued at its first processing.
  (b) A deployer infantry type given *unload* while undeployed, with a test file that removes the area-fire flag from its deploy
  weapon, once with its undeploy delay set to 30 and once to −1. Expected: it deploys and *guard*
  becomes current on that tick. In the first case the first processing of *guard* comes 30 ticks
  later and the unit then starts *undeploy* by itself (claim 31, first row), and its next
  processing comes exactly the frame count of its *undeploy* sequence (S-0107) after that one.
  In the second case the
  first processing of *guard* comes 450 ticks later and the unit stays deployed; *unload* again
  then gives *guard*, an *undeploy* sequence and the next processing 450 ticks later. (c) The type
  singled out by name, with its area-fire deploy weapon and an undeploy delay of −1, given *unload*
  while undeployed: expected the first processing of *guard* comes (frame count of its *deploy*
  sequence + 1) ticks later, and its target does not become its own cell. (d) An infantry unit
  ordered to enter a full transport: expected it falls back to rest. (e) An armed infantry unit
  that is not a building taker, of a human-controlled owner (so that the hunt rule does not apply),
  placed in *capture* without a target on the cell of an open gate of its own house, on flat
  ground with no other unit within 2 cells; repeated with the unit on an ordinary cell. Expected,
  on the gate: at its first processing it falls back to rest and on the same tick starts to walk to
  a neighbouring empty cell, the one that S-0208 claim 13 gives for the tick number of that tick.
  Expected, on the ordinary cell: it falls back to rest and stays on its cell. Falsified, case by case: (a)
  by anything other than *hunt* queued at the first processing; (b) by *guard* not current on the
  tick of the deploy, by a first *guard* processing other than 30 ticks later in the first run or 450 ticks later in the second, by no
  self-undeploy in the first run, by an interval after the self-undeploy other than the frame
  count of the *undeploy* sequence, or by an undeploy without a new *unload* in the second; (c) by a
  first *guard* processing at any other interval, or by the target becoming the unit's own cell; (d)
  by the unit staying in *enter*, or entering; (e) by no move from the gate cell, by a destination
  other than the one S-0208 claim 13 gives, or by a move from the ordinary cell. Runs: three each.
  (f) Deploy weapon slot (claim 30, Terms): named test file giving a deployer infantry type that is
  not the one singled out by name an undeploy delay of −1, an area-fire weapon in its first slot
  only (its second slot holding a weapon without the area-fire flag) and the spray-attack flag. The
  unit, undeployed, is given *unload*. Expected: on the tick of the deploy its target becomes
  its own cell, *guard* becomes current, and the first processing of *guard* comes exactly 450
  ticks later (claim 30, the row of an area-fire deploy weapon of any other type). Control: the
  same file without the spray-attack flag: expected its target does not become its own cell, and
  450 ticks again (the row of a deploy weapon without the area-fire flag). Second control: the
  area-fire weapon in the second slot only, without the spray-attack flag: expected its own cell as
  target and 450 ticks. Falsified by the own-cell target in the first control, by its absence in
  the main run or the second control, or by a first *guard* processing other than 450 ticks after
  the deploy in any of the three. Runs: three each.
- **B13 (claim 21, Import mapping).** A test map whose override section gives the *guard*
  behaviour's section a rate of 0.2 and nothing else. Expected: an armed building in *guard* shows intervals 180, 181 or 182 (the anti-air rate
  followed the new rate). With the override also giving an anti-air rate of 0.07, expected
  63, 64 or 65. With the override giving an anti-air rate of 0 instead: expected 180, 181 or 182
  (an explicit 0 also becomes the rate). Falsified by the building keeping the base rules'
  anti-air rate in the first or third case, or by an interval other than 63, 64 or 65 in the
  second.
  Runs: 100 intervals each.
- **B14 (claims 5, 12).** (a) A test map places a vehicle with the behaviour name for *queued move*,
  and another with the name for *attack move*. Expected for both: that behaviour current,
  processing every 450 ticks, no movement and no automatic search over 3000 ticks with an
  enemy in range. (b) With the attack-move gesture (S-0214), give an armed vehicle of a type
  eligible for attack-move an attack-move order on an empty cell 10 cells away; then give it one on
  an enemy unit out of its weapon range. Expected: after the first order *move* becomes current,
  with the cell as destination; after the second, *attack* becomes current with the enemy as
  target; in neither case does *attack move* become queued or current. Falsified by any movement,
  target search or other interval in (a), or in (b) by *attack move* in a slot or by the other
  behaviour becoming current. Runs: one each in (a), three in (b).
- **B15 (claims 13, 14 row 4 and 15 row 3).** (a) Guard-spot row: a human-controlled infantry unit
  given an area-guard order on a cell 5 cells away arrives there. Named test file: player scatter
  on. It is then damaged by an attacker it cannot fire back at, so that it scatters off the guard
  spot's cell (an order would clear the guard spot, claim 12). Expected, if the scatter ends with a
  fall-back to rest: the unit takes the guard spot as its destination and walks back, keeping the
  guard spot. (b) Route row and claim 15, row 3: with the route-mode input of S-0213 (claim 16),
  give an unarmed vehicle of the human-controlled house, in *move*, a planned route of three move
  points, each 6 cells from the last, and commit it. Expected: when it arrives at the first point
  and falls back to rest, the next point eventually becomes its destination without further player
  input, while the recorder shows its queued destinations empty. Record the route-generated order
  and its boundary of application: the next point is not assigned inside the fall-back. With no
  other destination, that fall-back queues *wait* with an immediate start before the issue (claim
  13), so for a vehicle that is ready to change (claim 11) *wait* is current until the later route
  order replaces it with *move*. For a vehicle that is not ready, *move* stays current and *wait* is
  queued, claim 15's row 1 does not match, and its rest table can queue *guard* in place of *wait*
  (claim 7), until the later route order replaces the queued behaviour with *move*. It then heads to
  the next point. (c) Claim 14, row 4: the same as (b) with an infantry unit of the human-controlled
  house, of a type that does not default to area guard and without the guard-area veterancy ability.
  Expected: as in (b), for the infantry unit with claim 14's table in place of claim 15's, except
  that its fall-back comes from claim 22's end-of-update check at the end of the update whose
  locomotion ends its walk at the first point, not from a later *move* processing. So at the end of
  that update a unit that is ready to change has *wait* current and no *guard* queued (claim 22),
  and one that is not ready has *move* current and *wait* queued, or *guard* in its place as in (b).
  The recorder reads at the end of each update. A run in which it shows a current behaviour other
  than *move* at the end of the update before that arrival update is invalid. Falsified in (a) by a
  unit that falls back to rest away from the guard spot and stays there; in (b) and (c) by an actor
  that never takes the next route point, by immediate assignment within the fall-back rather than an
  order boundary, or by continuation only through a non-empty queued-destinations list (that would
  be the queued-destinations row, not the route). (d) Guard-spot row outside *area guard*: the
  recorder also reads the guard spot. An armed infantry unit of the human-controlled house, of a
  type that does not default to area guard, is given an area-guard order on a cell 5 cells away and
  arrives. An enemy unit beyond its weapon range and within (*S* + 0.5) × 256 leptons (F7) fires at
  it, so that it overrides to *attack* (claims 10 and 18) and walks off the guard spot's cell
  towards the attacker. Once it is at least 2 cells from that cell, it is given the stop order
  (claim 12: target and destination cleared, guard spot kept, no revert). Expected: at its next
  *attack* processing, which falls back to rest, its destination becomes the guard spot's cell,
  *move* is queued, and the guard spot is empty. A run in which the recorder shows no guard spot, or
  a current behaviour other than *attack*, just before that processing is invalid. Falsified by a
  destination other than the guard spot's cell, by no *move* queued, or by a guard spot kept. Runs:
  three each. (e) Temporary driving: produce a stock teleporting vehicle at a stock ground vehicle
  factory with no rally point, recording locomotion, contact, behaviours and fall-backs as S-0417
  U6/U8 propose. Expected: factory exit uses temporary driving, and at the first eligible fall-back
  the recorder shows the active movement kind change from driving to teleporting. After release,
  order a distant clear cell: on the order's tick the movement kind changes back to driving (S-0207
  claim 28) and the vehicle drives there. No restoration at that fall-back, or a jump to the ordered
  cell, falsifies. A docking jump at a later refinery approach (S-0402) is a further sign that
  teleporting locomotion was restored. Three runs; no modified locomotor data is needed. (f) Rest
  with an existing destination: a named test map gives *move* the no-threat property, clears its
  zombie and retaliate properties, and turns player scatter off. Order a plain long move to a
  non-area-guard-default infantry unit and an unarmed vehicle, with no route, guard spot, scatter
  ability or queued destinations. An enemy damages each while it is still moving, leaving it alive.
  Record its damage-time fall-back, destination and slots. Expected: the destination survives and no
  resting *guard* is queued, because the destination row queues the already-current *move*. Loss of
  the destination or a queued *guard* on that fall-back falsifies the destination row's precedence.
  Three runs per kind; a run without the damage-time fall-back is invalid, not confirming evidence.
  (g) Rate after the route row (claim 6): a named test file sets the rate of *wait* to 0.06 and that
  of *attack* to 0.05, as in B2. With the route-mode input, give an armed vehicle of the
  human-controlled house, ready to change, a route whose first point attacks a building of a
  computer-controlled house and whose second point is a move point 6 cells further, and commit it.
  While the vehicle's *attack* is current, destroy the building with a second unit of the
  human-controlled house. Record the slots and the processing timer at the exit of the vehicle's
  first *attack* processing with no target, as B3 part e does. Expected: *wait* current at that exit
  and the delay set there one of those that claim 6 gives for 0.06, as in B2. Falsified by a delay
  that only 0.05 gives. Runs: 20.
- **B16 (claims 11 and 32, buildings).** (a) Record the processing intervals of an armed defensive
  building without the SAM flag while it attacks a stationary target in range, with a weapon whose
  rate-of-fire field (in a named test file) is 20 ticks or more: expected an interval of 1 tick
  after each processing at which it fires and 2 ticks after processings while its weapon rearms,
  never a rate delay. (b) An unarmed building whose type has the stupid-guard-mode flag: expected exactly
  100 ticks every time. (c) A SAM building of a house with full power attacking an aircraft in flight: expected every interval
  1 tick until the target is gone, then *guard* current at most 2 ticks after the target's loss
  (at the building's next processing if it was in *aiming*, one processing later if it was in
  *aimed*, claim 32). (d) An armed defensive building
  without the SAM flag that has been in *guard* for at least one full processing interval, with no
  target, is given a force-fire order on an enemy in range. Expected: *attack* becomes current at
  the building's first per-tick start after the order takes effect (its update on the next tick,
  S-0203 claim 7) and is processed on that tick; the time from the order to *attack* is at most 1
  tick, whatever is left of the anti-air delay. (e) Out of range: the building of case a attacks an
  enemy vehicle, which an order then drives out of the building's weapon range. The vehicle
  stands just inside the edge of that range, and the order is given on the tick of a recorded
  shot, so that the vehicle leaves range while the rearm countdown runs. The read-only recorder
  records, at each of the building's processings, the firing check's answer, its target and its
  current mission. A run is judged only if at least one processing after the vehicle leaves range
  records the answer *rearming* (S-0301 claims 9 and 14); any other run is recorded and not
  judged. Expected, in a judged run: at each processing after the vehicle leaves range whose
  recorded answer is *rearming*, the building keeps its target; at the first processing after
  the vehicle leaves range whose recorded answer is *out of range*, the target is empty and
  *guard* is current. Falsified by *attack*
  becoming current only at the building's next scheduled *guard* processing (a wait that grows
  with the remaining anti-air delay), by an interval other than 1 or 2 ticks in case a, by
  1-tick intervals throughout a rearm period or a 2-tick interval right after a shot in case a,
  by other intervals in case b, by a target dropped at a processing whose recorded answer is
  *rearming*, or kept at the first one whose answer is *out of range*, in case e, or in case c by an
  interval other than 1 tick or by *guard* becoming current more than 2 ticks after the target's
  loss. Runs: 100 intervals
  each in cases a to c, five orders in case d, given at different points of the anti-air delay,
  three judged runs in case e, repeating a run that is not judged, with at most ten attempts in
  all; if fewer than three runs are judged after ten attempts, case e is reported as not run,
  not as confirming or falsifying evidence. (f) Unarmed delays: named test file giving *guard* the rate 0.07
  (*D*<sub>r</sub> = 63). An unarmed building of the human-controlled house that is neither a
  factory nor a repair bay nor of the stupid-guard type (for example a power building), and a
  unit repair building with no vehicle on it. Expected: intervals 189 to 191 (3 × 63 + *U*(0, 2)) for
  the first and 63 to 65 for the second, each value seen. Falsified by 63 to 65 for the first or
  189 to 191 for the second. Runs: 100 intervals each. (g) Armed *guard* building: an armed
  defensive building without the SAM flag in *guard* with no target; an enemy unit walks into its
  range and the building's per-tick search takes it as target. Expected: at its next *guard*
  processing *attack* is current on that tick, and the next processing comes 1 tick later (not an
  anti-air delay). Falsified by *guard* staying current with an interval of the anti-air delay.
  Runs: five. (h) Turret snap (claim 32, F8): named test files give an armed defensive building
  type without the SAM flag the turret flag, a turn-rate field of 128 (so *w*₈ = −128 and the
  bound covers every aim error) and a weapon whose rate-of-fire field is 20 ticks or more, and
  set its own type's rules voxel-turret flag, supplying a voxel turret model in the test files.
  A run in which the turret image fails to load is invalid. A stationary enemy vehicle is placed in range in the
  direction opposite to the turret's facing, and the building is given a force-fire order on it.
  Expected: at its first *attack* processing it fires (the shot is recorded on that tick) and the
  next processing comes 1 tick later, with no 2-tick interval before the first shot. Control: the
  same files with the rules voxel-turret flag cleared: expected at least one 2-tick interval
  (rotating) before the first shot. In both cases repeat with conflicting voxel-turret
  settings in the image art, changing only those art settings as a negative control:
  neither the snap nor the processing intervals change. Also compare an art-only flag
  with the flag omitted from every applicable rules layer in a fresh game: both use the
  initial rules default and do not snap.
  Falsified by a 2-tick interval before the first shot in the main run, by none in the
  rules-cleared control, or by any art-only change affecting the result. Runs: five each.
  (i) *Still turning* (claim 32, the row of other answers): the control of (h), with a turn-rate
  field small enough that the half turn takes at least 20 ticks (S-0206), and the recorder also
  reading the turret facing. Expected: the first *attack* processing (answer *not facing*) is
  followed 2 ticks later by the next; every later processing at which the recorder shows the
  turret facing still changing (answer *still turning*, S-0301 claim 9) is followed 1 tick later
  by the next, until the first shot. Falsified by a 2-tick interval after a processing at which
  the turret is still turning, other than the first. Runs: five.
  (j) SAM power row: a test map gives the bystander (named test objects) a SAM building whose
  type needs power and drains power, and one power plant of a test type that no house can build,
  so that it has full power. Short of power is house-wide, and the bystander may build power
  plants at its own base, so the named test file gives the SAM type a drain larger than the
  power output the bystander's own buildings can reach within the run, and the test power-plant
  type an output that covers the whole drain. The recorder also reads whether the bystander's
  house is short of power. A named test file gives an aircraft type of the human-controlled house that can stand
  landed a strength high enough to survive the test. The aircraft flies into the building's range.
  A timed map trigger destroys the power plant, so that the house is short of power, at a time
  chosen so that the building is then attacking the aircraft, and the aircraft is then ordered
  to land on open ground within the building's range. Expected: after the first processing that
  follows the loss of power, no shot is fired, every processing interval is 1 tick, and *attack*
  stays current for as long as the house is short of power, also after the aircraft has landed
  (world height 0), whether or not the building still holds it as target. Control: the same with
  a second test file that removes the needs-power flag from the SAM type: after the aircraft has
  landed, *guard* is current at most 2 ticks later. A run in which the building is not in *attack*
  when the power plant is destroyed is invalid, and so is one in which the house is not short of
  power after the plant is destroyed, or stops being short of power before the observation ends. Falsified by *guard* current while the house is
  short of power, or by a shot after the first processing that follows the loss of power, in the
  main run; or by *attack* still current more than 2 ticks after the landing in the control. Runs:
  three each.
  (k) SAM aiming tolerance: a named test file gives the SAM building type a small turn-rate field
  and the aircraft type a high speed. The recorder also reads the building's progress count, the
  facing it rotates towards its target, whether that facing is rotating, and the direction to the
  target (S-0204). Aircraft of the human-controlled house pass close to the building, which has
  full power. Take every processing in *aiming* (count 0) at which the facing is not rotating, the
  target is suitable, and the aim error ε (as in F8: the difference between the facing and the
  direction, taken the shorter way round) is known. Expected: the count is 1 (*aimed*) after that
  processing if and only if ε ≤ 8192 facing units. Falsified by *aimed* after an ε above 8192, or
  by *aiming* kept after an ε of 8192 or less. Runs: until 30 such processings with ε above 8192
  and 30 with ε at most 8192 are recorded.
  (l) SAM firing both weapons: a named test file gives the SAM building type a secondary weapon
  that can hit aircraft and whose shots the recorder can tell from the primary's (for example a
  different projectile type). With full power it attacks aircraft in flight. Expected: on every
  tick on which it fires, one shot of each weapon. Control: the same type without a secondary
  weapon: one shot. Falsified by a firing tick that shows only the primary weapon's shot, or only
  the secondary's, in the main run. Runs: 20 firing ticks.
  (m) Out-of-stage rule: named test files give a gate type that the human-controlled house can
  build the SAM flag and a weapon that can hit aircraft, and give *attack* an anti-air rate of 0.2
  (*D*<sub>aa</sub> = 180). The human player builds the gate and walks an infantry unit through
  it, so that the gate is in *open*. An aircraft of the enemy bystander (a named test object) attacks the gate (for
  example one that the test map places in *hunt* with the gate the only target within its reach). The
  recorder also reads the gate's progress count. Take a run in which the gate retaliates (an
  override to *attack*, the suspended slot showing *open*) while its count is 2 to 5. Expected:
  for as long as that *attack* stays current, every processing interval is 180, 181 or 182 ticks,
  the count does not change, and the gate fires no shot. A run in which the override comes while
  the count is 0 or 1 is set aside (the stages of claim 32 then apply); if no run gives an override
  at all, the case is recorded as not applicable. Falsified by a 1-tick interval, a change of the
  count or a shot during such an *attack*. Runs: until three runs are not set aside, at most 20.
- **B17 (claim 25).** A human-controlled armed vehicle placed in *sticky* (test map), player
  return fire off, with an enemy unit driven into its weapon range and firing at it: expected no
  target taken over 2000 ticks. The same with a computer-controlled vehicle (the original-only
  part of claim 25, a note for the AI designers): expected a target only through retaliation (the
  suspended slot shows *sticky*). Falsified by a target taken otherwise.
  Runs: three.
- **B18 (claim 2).** Named test file: rates of *move*, *guard* and *area guard* 0.001 (held as a
  single just above 0.001, so *D*<sub>r</sub> = 0; an actor in *move* or *guard* then has a
  processing tick at least every 2 ticks, and one in *area guard* at least every 5).
  (a) Chrono move: an armed vehicle of the human-controlled house in *guard* is moved by the
  chronosphere. Expected in the original: during its leaving stage the processing timer's expiry
  does not move forward and target and destination are empty, unless a pending point suppression
  keeps the destination (claim 2; this scenario sets none). Record due behaviour actions
  separately on pending, departure, retry, placement, arrival-effects and arrival-hold turns,
  including hold expiry. Original continuation on those turns remains unmeasured under
  S-0513 claim 48; use its scenario SW23 parts a to e to distinguish the alternatives rather
  than expecting processing throughout phase-in. In our engine, SW23 part f tests claim 51's
  design contract: no ordinary processing or per-tick behaviour start from the first pending
  update through arrival effects, then ordinary eligibility for both after each arrival-hold
  expiry test, including while still holding. Extend this check with SW23 part g's queued
  behaviour and readiness observations. An ordinary processing or per-tick start in an
  excluded turn, or refusal solely because of arrival hold after that test, falsifies the
  engine contract. Separately, order an
  armed non-harvester vehicle, given teleporting locomotion by the
  named test map, to a clear cell 10 cells away. Expected: no leaving stage and no corresponding
  processing exclusion; its jump follows S-0207 claim 20 d, and its phase-in lasts F9 of that
  spec. During phase-in processing continues at least every 5 ticks, whichever of *move*,
  *guard* or *area guard* is current. (b) Erasing: the same vehicle without a chrono move is locked onto by an erasing
  weapon; expected no processing tick from the lock-on until it is removed (or let go), and target
  and destination empty on those ticks, unless a pending point suppression keeps the destination
  (claim 2; this scenario sets none). (c) A test map with a tunnel: an infantry unit and a
  vehicle ordered through it show no processing tick while inside. Falsified by a processing tick
  during the leaving stage, while being erased or inside the tunnel, or by a gap of more than 5
  ticks between processing ticks during a locomotor jump's phase-in. The original's superweapon
  continuation gaps are not covered by that five-tick expectation. (d) Jumping retaliation target: an armed
  observer first retaliates against an enemy non-harvester teleporting vehicle, then that vehicle
  is moved by a test-map team to another clear cell. Record observer target, suspended behaviour
  and slots on the jump tick. Expected: override reverts on that tick and the jumping vehicle
  is not retained as target. Repeat with an ordered attack rather than retaliation: target drops
  without a revert. Retained target or a revert postponed to phase-in end falsifies the rule.
  Erasing weapons are excluded. Runs: three each.
- **B19 (claim 31).** (a) A computer-controlled deployer infantry type that deploys to fire, not
  immune to radiation, with a negative undeploy delay, placed in *guard* on its guard spot without
  a destination, once for each computer entry of the setup screen (hard, normal and easy computer,
  difficulty indexes 0, 1 and 2, Terms), with a named test file giving the auto-deploy delay list
  three different entries: expected it starts its *deploy* sequence at the first
  processing after (start tick + the list entry for that index), and that processing's delay
  equals the *deploy* sequence's frame count. (b) The same type deployed: expected intervals
  *D*<sub>r</sub> + 0, 1 or 2. (c) A type that deploys to fire and is immune to radiation (named test
  file giving its secondary weapon a radiation level *W*), deployed on a clean cell: expected it
  fires at its own cell and the interval equals its *deployed-fire* sequence's frame count; deployed
  on the centre cell of a field whose current integer field level is at least trunc(*W* / 3):
  expected intervals *D*<sub>r</sub> + 10 to 20, each value seen. Record centre, current field
  level and cell exposure separately (S-0525 claim 7). Repeat with exposure from a neighbouring
  centre only, then with residue after an ended field, and no field centred on the unit's cell:
  expected the same own-cell firing as the clean-cell case when its firing check permits it.
  Suppression solely by neighbouring exposure or residue falsifies the centre-field test.
  (d) The firing rules refuse: the unit of (c) deployed on a clean
  cell, with a named test file giving its weapon a rate of fire longer than its *deployed-fire*
  sequence, so that its weapon is still rearming at the next processing. Expected: at that
  processing it drops its target and the interval is *D*<sub>r</sub> + 10 to 20, each value seen.
  Falsified by a deploy before the list delay, a deploy that follows the entry of another index
  (for example the easy computer's unit deploying after the first entry), other intervals, or a
  draw outside 10 to 20. Runs: three per computer entry in case a, 100 intervals each in cases b to d.
  (e) Timing control for Q-0983: on a small clear flat skirmish map, deploy a stock
  radiation-immune infantry unit that deploys to fire, with negative undeploy delay,
  on a cell without a centred field. Keep enemies out of its acquisition range and
  hold game speed fixed, with no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Use the measurement setup's
  read-only recorder and tick-aligned capture to record the deciding processing tick,
  shot, target before and after processing, sequence start and field creation or top-up.
  Expected: the secondary shot occurs in that processing, the own-cell target is absent
  at its end, and immediate detonation creates or tops up radiation on that same tick.
  Repeat as a separately named data-only test with standing, prone and secondary
  fire-frame fields all later than frame zero, then change those fields without
  changing the deployed-fire frame count. Expected: the shot remains immediate and
  the next processing delay remains that frame count in ticks. No second shot from
  this presentation sequence is expected without a new firing decision or target.
  A shot postponed to any fire frame, target retained until that frame, or a delay
  changed solely by those fields falsifies the claim. Tool: proposed read-only
  tick recorder plus scripted deploy input, 800×600 capture; no harness change is
  performed here. Runs: three stock and three per named-data variant.
- **B20 (claim 20).** Skirmish: a computer-controlled house with armed units in *guard* near its
  base and not in a team. An enemy attacks a base building out of those units' own range. With
  stock rules, record which units the computer sends to defend; then with a named test file that
  turns the recruitable property of *guard* off, repeat. Expected: with the property off, no unit
  in *guard* is sent (units can still act on their own targets). Falsified by a *guard* unit sent
  to defend with the property off. Its own void rule: the units the computer player may send are
  the subject, so the bystander's units entering the area around its base do not void a run. The
  named test objects are the attacked base building and the units in *guard* at the moment of the
  attack, and only these are observed. A run is void only when the bystander is defeated before the
  observation ends. Runs: five each.
- **B21 (claims 6 and 24, enter-nearest instructions).** Named test file: rate of *guard* 0.07
  (*D*<sub>r</sub> = 63) and rate of *capture* 0.05 (*D*<sub>r</sub> = 45).
  (a) Test map: a computer-controlled house with one infantry unit of an occupier type (so the
  hunt rule of claim 28 does not apply to it), in a team whose script tells its members to
  garrison the nearest building, and two empty neutral buildings that the unit may occupy, A
  nearer and B farther. While the unit walks to A, a map trigger destroys A (or a unit of another
  house occupies it first). Expected: the unit falls back to rest, rests in *guard* as a team
  member (claim 14, row 12) and, at a later *guard* processing with no new action of its team script, takes
  B as its destination with *capture* current on that tick; the first *capture* processing comes
  exactly 45 ticks after that *guard* processing (*capture*'s *D*<sub>r</sub>, no spread). A run in
  which, on the tick of that *guard* processing, *capture* is queued but not current (the unit was
  not ready to change, for example because it was firing or in a sequence that is not
  interruptible, claim 11) is set aside; at least three of the five runs must not be set aside.
  (b) The same, but the trigger destroys both A and B while the unit walks to A. Expected: the
  unit stays in *guard*; the *guard* processing at which it finds no building is followed exactly
  63 ticks later by the next, and from then on it shows the general *guard* intervals 63, 64 or
  65, each value seen.
  (c) Absorbing building, if the team scripts offer an instruction to enter the nearest absorbing
  building (otherwise the case is recorded as not applicable): the same house owns two absorbing
  buildings that accept the unit, A nearer and B farther, and a team script gives the unit that
  instruction. While it walks to A, a map trigger destroys A. Expected: at the *guard* processing
  that finds B, B becomes the destination and *enter* is queued while *guard* stays current on
  that tick; *enter* becomes current at a later per-tick start (the next tick if the unit is ready).
  Falsified in (a) by the unit not heading for B, or by an interval of 63, or any interval other
  than 45, in a run that is not set aside; in (b) by an interval other than 63 after the
  processing that finds nothing, or by intervals of exactly 63 throughout; in (c) by *enter*
  current on the tick of the *guard* processing that finds B. Runs: five each.
  (d) As (c), with a second absorbing building of the same owner that is nearer to the unit than B,
  and a second unit of the same owner already in contact with it (so the first unit, in *guard*, finds that
  building held and waits in line for it, S-0605 claim 3). Expected: at the *guard* processing that
  finds that building, the first unit queues no second *enter* for it, and its current behaviour stays
  *guard*. Falsified by any *enter* or *capture* queued or made current at that processing. Runs: five.
  Setup not yet established: whether a team script can hold the second unit in contact is for the owner to
  confirm; if it cannot, record the part as not applicable, as (c) does.
- **B22 (claims 5 and 6, shared processings and their rates).** (a) Named test file: rate of *guard* 0.2
  (*D*<sub>r</sub> = 180) and rate of *sticky* 0.07
  (*D*<sub>r</sub> = 63). A test map places an armed vehicle of a human-controlled owner in
  *sticky*, with no enemy within sight. Expected: its processing intervals are 63, 64 or 65, each
  value seen, and never 180, 181 or 182. (b) Named test file: rate of *capture* 0.2 and rate of *sabotage* 0.07. An infantry
  unit of the human-controlled house whose type has the explosive-charges flag is given the
  sabotage order on an unarmed enemy building 15 cells away whose type accepts charges (S-0214
  turns it into *sabotage* with the building as destination and no target). Expected: while it
  walks there, *sabotage* is current and its processing intervals are 63, 64 or 65, each value
  seen. Falsified in either case by intervals of 180, 181 or 182 (the rate of the behaviour whose
  processing is shared), by 450 (no processing rule), or by any other interval while the
  behaviour stays current. Runs: 100 intervals per case; in case b, as many walks as
  needed, each with a fresh unit and building placed by the test map.

- **B23 (claim 11, vehicle readiness).** (a) Queued *enter*: a vehicle of the human-controlled house
  in *move* on a long route, damaged, is ordered to enter a unit repair building. Expected: *enter*
  becomes current at the vehicle's first update after the order takes effect, while the vehicle
  is still moving (the exception for a queued *enter*). Control: an attack order on a distant enemy
  building given in the same state stays queued while the vehicle moves (B3, case b). (b) Contact
  with a vehicle-factory building: a vehicle that a factory has just produced, given an attack
  order on an enemy in range while the recorder shows it in contact with the factory. Expected: the
  order stays queued until the contact ends, and then *attack* becomes current at the vehicle's
  next per-tick start. A second new vehicle given a move order in contact becomes *move* at its
  first update after the order takes effect. Falsified by *attack* current while the contact
  lasts, by *enter* staying queued while the vehicle moves in (a), or by *move* staying queued
  in (b). (c) Letting passengers out: a transport vehicle of the human-controlled house that
  carries five infantry units stands still and is ordered to unload them. While the recorder
  shows *unload* current and at least one passenger still inside or still leaving, give it a move
  order to a cell 10 cells away. Expected: *unload* stays current as long as a passenger is still
  leaving, with *move* queued (or replaced in the queue by the *guard* that the end of the
  unloading queues, transport specs); *move* or *guard* becomes current only after the last
  passenger has left. Falsified by *move* current while a passenger is still inside or leaving. A
  run in which the order system cancels the unloading at once is not applicable. Runs: three each.
- **B24 (claims 23, 24 and 26: distributed fire, charge targets, hover-attack cell).** (a)
  Distributed-fire row: a named test file sets the distributed-fire flag on an armed vehicle
  type and the rate of *guard* to 0.07 (*D*<sub>r</sub> = 63). The vehicle of the human-controlled
  house stands in *guard* with three enemy units in its weapon range. Expected: at a processing
  at which the recorder shows the candidate list non-empty and the weapon is not rearming,
  the next processing comes 1 tick later and the processing tick takes no more raw values than
  the comparison ticks (no draw for the delay); with an empty list and no rearm time remaining
  the interval is 63 to 65. A processing while the weapon is rearming instead chooses its
  remaining rearm time (claim 24), and does not test the distributed-fire row. For processings
  without rearm time remaining, falsified by an interval other than 1 tick after a non-empty
  reading, an interval outside 63 to 65 after an empty reading, or an extra delay draw with a
  non-empty list. A run with no non-empty reading while the weapon is not rearming leaves this
  row untested. Runs: 100 processings. (b) Charge target: an infantry type whose
  secondary weapon charges friendly buildings stands in *guard*, with no target, next to a friendly
  building that can be charged on one of the 8 cells around it; repeat with the unit in *area
  guard*. Expected: at its first processing the building is its target and *attack* is queued
  (current at its next per-tick start). Control: the building two cells away: expected no target
  and no *attack* queued. Falsified by no charge target next to the unit, or by one two cells
  away. Runs: three each. (c) Charge-target check: the unit of (b) with *attack* current and the
  building as its charge target; an enemy unit then enters its weapon range. Expected: at its next
  *attack* processing the enemy is its target, and the unit approaches or engages it in that same
  processing. Control: an enemy beyond the weapon range and within sight: expected the building
  stays the target. Falsified by the building staying the target with an enemy in weapon range.
  Runs: three. (d) Hover-attack cell: a named test file sets the hover-attack flag on an armed
  ground vehicle type (its height above the surface is 0). The vehicle of the human-controlled
  house is ordered to attack a stationary enemy building in its range, and, in another run, stands
  in *guard* while an enemy building is in range (target present). Expected: at the processing, the
  recorder shows a destination on a cell other than the vehicle's own cell; the cell itself belongs
  to the movement specs. Falsified by an empty destination at 20 processings in a row. Runs: three.
- **B25 (claim 22, deployed infantry).** (a) Human-controlled owner: a deployer infantry type that
  deploys to fire, deployed, is given a move order. Expected: *move* becomes current, at its first
  processing the destination is cleared and the unit stays deployed, and the next processing comes
  1 tick later. A run in which the order system undeploys the unit first is not applicable.
  Falsified by a kept destination, or by an interval of the standard delay. Runs: three. (b)
  Computer-controlled owner (the original's only): a test-map team whose script moves a deployed
  computer-controlled deployer infantry unit, once with a negative undeploy delay and once with an
  undeploy delay of 0 or more (named test file). Expected: with the negative delay the unit starts
  its *undeploy* sequence and the next processing comes after exactly the sequence's frame count.
  With 0 or more the general rule applies (interval 1 tick if it has no destination, else the
  standard delay). Falsified by other intervals. Runs: three each.
- **B26 (claims 24 and 26, the original's only: computer-controlled owner).** (a) *Guard*
  sabotage: a computer-controlled infantry unit with explosive charges stands in *guard* and
  takes an enemy building in its range as target. Expected: at its next *guard* processing
  *sabotage* is queued (current at its next per-tick start), unless it is already current. (b)
  *Area guard* row 5: the same unit placed in *area guard* with a building as target. Expected: at
  its processing *sabotage* is queued and the interval is exactly 1 tick; with a human-controlled
  unit of the same type, nothing is queued. (c) Guard spot on an allied building: a
  computer-controlled armed unit placed in *area guard* with its guard spot on a cell that holds an
  allied building. Expected: after its first processing the guard spot is the nearest free cell
  (for the unit of a human-controlled owner it stays on the building's cell). Falsified by no
  *sabotage* queued in (a) and (b), by an interval other than 1 tick in (b), or by a guard spot
  that does not move in (c). Runs: three each.
- **B27 (claims 6, 26 and 33: order of draws and of processing and search within a tick).** Setup as
  the common setup, but the recorder reads the full state of the scenario random stream at the
  start of every tick (the random-state recorder of the measurement setup), not only its
  position. From that state, S-0202's generator and its reduction to a range predict, for each
  tick, the values that the tick's ranged draws would receive in either of the two orders under
  test. A tick counts as valid when the stream advances by exactly the raw values that the draws
  named in the case take under at least one of the two orders; any other tick is set aside, since
  another rule drew on it. A valid tick whose advance fits only one order counts as evidence for
  that order. Each case uses only valid ticks on which the two orders give different readings.
  - (a) Processing before search (claim 33, part d): named test file giving *guard* an anti-air
    rate of 0.2 (*D*<sub>aa</sub> = 180). An armed defensive building of the human-controlled
    house stands alone in *guard*, with no object of another house within its reach, so that its
    per-tick search finds no candidate and takes only the timer's *U*(0, 2). On a tick on which
    both its processing and its per-tick search run (its processing timer and its targeting timer
    run out on the same tick), with predicted values *u*₁ then *u*₂: expected processing delay
    180 + *u*₁ and targeting-timer restart *N* + *u*₂ (*N* the normal targeting delay). Falsified
    by 180 + *u*₂ and *N* + *u*₁ on valid ticks. Runs: until 30 valid ticks.
  - (b) The search follows the processing's result (claim 33, part b): the building of B16, case
    e, in *attack* for longer than the normal targeting delay + 2 ticks (so its targeting timer has
    run out: *attack* runs no per-tick search), and its target driven out of range. The processing
    that drops the target is the one case e judges: the building's first processing after the
    target leaves range whose recorded firing-check answer is *out of range* (claim 32), not
    necessarily the first processing after the target leaves range.
    Expected: on the tick of that processing, which drops the target and starts *guard*, the
    recorder shows a restart of the targeting timer on that same tick. Falsified by the first restart only on a
    later tick. Runs: five.
  - (c) A target found by the search waits for a later processing (claim 33, part c): named test
    file giving *guard* an anti-air rate of 0.001 (*D*<sub>aa</sub> = 0, so the building processes
    every 1 or 2 ticks and most acquisitions fall on a processing tick). An armed defensive
    building without the SAM flag in *guard*, with no target; enemy units walk into its range one
    at a time. Over the runs, take the ticks on which the recorder first shows a target
    found by the per-tick search and the building also processes on that tick. Expected: that
    processing chooses the anti-air delay (it does not start *attack*), and *attack* becomes
    current at a later processing. Falsified by *attack* current on the tick on which the target
    first appears. Runs: until five such ticks.
  - (d) The delay's draw after the search's (claim 6): named test file giving *area guard* the
    rate 0.07 (*D*<sub>r</sub> = 63). An armed vehicle of the human-controlled house stands in
    *area guard* on its guard spot, with no object of another house within reach, so that the
    search of its processing finds no candidate and its idle action takes no draw. On a
    processing at which its targeting timer has run out, the predictor gives the readings of both
    orders: (*t*, *u*) when the timer's *U*(0, 2) is drawn first and the delay's *U*(1, 5) second,
    and (*t*′, *u*′) for the other order. Expected: targeting-timer restart *N*<sub>a</sub> + *t*
    (*N*<sub>a</sub> the area-guard targeting delay) and processing delay 63 + *u*. Only ticks on
    which the two orders give different readings count. Falsified by *N*<sub>a</sub> + *t*′ and
    63 + *u*′ on valid ticks. Runs: until 30 valid processings.
  - (e) An actor with an attack-move memory (claim 33, its paragraph on such actors): named test
    file giving *move* the rate 0.2 (*D*<sub>r</sub> = 180). An armed vehicle of the
    human-controlled house walks back and forth between two cells 30 cells apart under
    attack-move orders, across open ground with no object of another house within its reach, so
    that each of its attack-move searches finds no candidate and takes only the timer's *U*(0, 2).
    On a tick on which both its *move* processing and its attack-move search run, with predicted
    values *u*₁ then *u*₂: expected processing delay 180 + *u*₁ and targeting-timer restart
    *N* + *u*₂. Falsified by 180 + *u*₂ and *N* + *u*₁ on valid ticks. Then an enemy vehicle of the
    bystander (a named test object), in *sleep* with the no-threat property of
    *sleep* turned off as in B10, case k, is placed by the test map beside the route. Expected: the
    first processing of *attack* comes on a later tick than the tick on which the recorder first
    shows that vehicle as the target. Falsified by a processing of *attack* on that tick. Runs:
    until 30 valid ticks, and five approaches.
  - (f) No draw for the 1-tick delay of *area guard* after a search that finds a target (claim
    26): the setup of B10, case k. The searcher belongs to a player house, so its search takes no
    draw for the objects it tests or scores (S-0309, claim 12) and only the timer's *U*(0, 2).
    Expected: on the tick of the processing whose search first finds the enemy, the stream
    advances by exactly the raw values of one ranged draw *U*(0, 2), and the targeting-timer
    restart shows the value it took. Falsified by an advance of two ranged draws, the second one
    a *U*(1, 5) or *U*(0, 2) of the delay. A tick on which the advance fits neither does not
    decide. Runs: 20.
- **B28 (claims 35 and 36: gates; the area-guard threshold default).** Named test file for cases a
  to d: rules giving the test gate type a deploy time of 0.1 (*T*<sub>d</sub> = 90 ticks) and a
  gate close delay of 0.2 (*T*<sub>c</sub> = 180 ticks), and *open* a rate of 0.05
  (*D*<sub>r</sub> = 45). The test map: a corridor one cell wide, closed across by a map-placed
  gate of the allied bystander, which is on the human-controlled house's team in the game set-up
  (so the two are allied from the start, and the gate rows of S-0209 give the human player's
  units *closed friendly gate*), with no other way round. The enemy bystander is on another team,
  so that the game has two sides. The gate is a named test object of the allied bystander, and
  the observed area includes the whole corridor. Besides the common void rules, a run is void when
  an object of either bystander that the scenario does not name enters the corridor or a cell of
  the gate, since an allied unit passing through would open the gate, or when the allied
  bystander sells the gate (an order to a named test object, rule b). A map-placed structure of the
  human-controlled house itself would not be placed in a skirmish (S-0009). Besides the common recorder, the
  recorder reads the gate's drawn stage on every tick and the tick on which each unit first
  enters and last leaves a cell of the gate's foundation.
  - (a) An infantry unit of the human-controlled house is placed on the near side and ordered to
    the far side on the first tick. Expected: the gate's drawn stage is the closed one until the
    unit reaches the gate (the gate starts closed); the gate's behaviour becomes *open* on the
    tick on which the unit first gets the entry outcome *closed friendly gate* (S-0209) for a cell
    of the gate; the drawn stage reaches fully open 90 or 91 ticks
    after that (the door time, plus at most the tick before the first processing), and the unit
    enters the gate's cells no earlier than that, after the wait that S-0209 gives a unit at a *closed friendly gate*. A
    second run with a deploy time of 0.2 must show the fully-open stage 90 ticks later than the
    first. Falsified by a gate that is open at the start, by the unit passing without waiting, or
    by a door time other than 90 (or 180) ticks, give or take 1.
  - (b) After the unit of (a) has left the gate's cells, record the tick on which the door starts
    moving towards closed (the recorder reads whether the door is moving or at rest, as in case
    f). Expected: no earlier than 180 ticks after the tick on which *open*
    started (the countdown counts from the opening start, not from the door being open), and no
    later than 180 + 2 × (45 + 2) ticks after the later of that tick and the tick the unit left the
    cells (a *waiting* processing while the unit still stood in the cells may have restarted the
    countdown; then one Std may pass before the *waiting* processing that finds the countdown run
    out and the cells empty, and one more before the *closing-start* processing, claim 35, c); the
    processing ticks of *open* in stage *waiting* are 45 + *U*(0, 2) apart, one draw each. Falsified by a closing start before 180 ticks, or one later than the bound.
  - (c) As (a), but a second unit of the house is ordered onto a foundation cell of the gate and
    stays there. Expected: the gate does not start closing while the unit stands there, and starts
    closing within 180 + 2 × 47 ticks after it leaves (as in b). Falsified by a closing start while the cell is
    occupied.
  - (d) As (b), and a second infantry unit of the house is ordered through the gate so that it
    reaches the gate 30 ticks after the closing starts. Expected: the drawn stage jumps to fully
    open on the tick after the unit first gets the entry outcome *closed friendly gate* for a cell
    of the closing gate (or on that tick), and the unit
    enters the gate's cells within 2 ticks of that. Off-switch result for comparison: the stage
    retraces over 30 ticks. Falsified by a door that takes 30 ticks or more to reopen.
  - (e) Area-guard threshold default (claim 36 a): named test files that leave the area-guard
    threshold key out of every rules layer and set the maximum skill level of the AI settings to 3
    (run L3) or to 4 (run L4). Skirmish. Both runs are played in a session in which no earlier
    game had a rules layer that gives the key (for example with the test files in place from
    program start), because a game-wide key keeps the value an earlier game of the session left
    (Import mapping; S-0006, claim 10). An armed vehicle of the enemy bystander (a named test
    object) placed in *move* with no destination, as in scenario B4. Expected: *guard* in L3, *area guard* in L4.
    Falsified by the same result in both runs, or by the reverse. A run in which the recorder shows
    another skill level for the enemy bystander's house is invalid. Runs: one each.
  - (f) Gate time that wraps (F9): as (a), with a named test file giving the gate type a deploy
    time of 2 400 000 game minutes (an integer, so held exactly in single precision; times 900 it
    is 2 160 000 000, above 2³¹ − 1, so the wrap of F1 gives a negative door time, which behaves as
    0). For this case the recorder also reads whether the gate's door is moving or at rest, and
    in which position (the drawn stage is not used, since its drawing from a negative door time
    belongs to the rendering topic). Expected: the door is at rest open at most 2 ticks after the
    gate's behaviour becomes *open*, and the unit then passes. Falsified by a door that is still
    moving, or still closed, more than 2 ticks after *open* becomes current, which would mean a
    conversion that saturates or keeps the large value.
  - (g) Where within the gate's turn the door reaches rest (claim 35 d, Q-0030): as (a), with the
    recorder also reading whether the door is moving or at rest, as in case f, and the position of
    the scenario random stream at the start and end of every tick (common setup). Record the tick
    *t* on which the door starts opening, the tick *r* on which it is first at rest open, and the
    first processing tick of the gate after *r* whose delay is a Std (the processing timer's
    expiry moves 45 ticks or more ahead, and the tick takes the Std's draw). Expected: *r* = *t* +
    90, and that processing comes in tick *r* + 1, the processings from *t* + 1 to *r* each having
    a delay of 0 (expiry on that processing tick, processed again at the next turn,
    no draw from the gate). Falsified by that processing in
    tick *r* + 2 (the door reaching rest after the processing in its turn), or in tick *r* or
    earlier.
  - (h) Drawing progress (claim 35 d1, Q-0685): repeat (a–b) at game speed index 2,
    first with stock art and then with a named art test file setting only the stage count to
    7 stages, retaining the 90-tick door time. Record the tick of each motion start, direction,
    rest state, displayed body-image index and damage state at every tick, including both
    motion-start ticks and the first stationary ticks. Use tick-associated screenshots with
    the read-only recorder; ordinary screen refresh may omit ticks. Compare the selected
    images to d1's two rounded floating-point operations, not exact rational division.
    Expected: opening starts at position 0, closing at displayed position 6 in the named
    test, and every later position follows d1; at elapsed 45 ticks the positions are 3 opening
    and 4 closing. Rest open while *open* is current still displays position 6. Repeat after
    nonlethal damage crosses the gate's damaged threshold: only S-0108 claim 4's damaged
    image addition changes. Falsified by another position at any recorded tick, by position
    7 being displayed, by reversing the opening expression without the closing subtraction,
    or by the ordinary closed image at rest open while *open* remains current.
  - (i) Reversal image (claim 35 d2): repeat (d) with the recorder also associating each
    displayed image with the tick on which the gate's processing reverses the door, rather
    than only the unit's entry attempt. Expected: displayed position *G* − 1 already in that
    reversal tick, while the door is still opening, and the same position at rest open after
    its next turn. Any lower image after reversal falsifies the original-side prediction.
    Test the off-switch retracing expression separately against our engine, not the oracle.
  - (j) Zero-time motion and skipped images, claim 35 d1/d3: repeat (h) with a named rules test file
    setting the deploy time to 0, leaving the art-stage count at 7. Record the draw selected
    after each start processing and before the next gate turn. Expected: opening is still
    moving with displayed position 6, closing is still moving with position 0; both are at
    rest on their next turns. No intermediate image is selected. Falsified by a different
    start-tick image or by immediate rest in the start processing. Separately set the deploy
    time to 0.002 game minutes, giving a 1-tick duration after F9's truncation, confirmed by
    the recorder: the only moving opening position is 0, rather than a forced visit to
    position 6. These named test data are measurement-only, with all other mods
    and external automated players off as in the common setup.
  - Runs for cases a to d, f to j: three each.

- **B29 (claims 14 and 15: rows and exceptions reached through other fall-backs).** Named test file
  for cases a and b: *guard* and *area guard* get the no-threat property, not the zombie property,
  and lose the retaliate and the scatter properties, so that damage from a known attacker makes the
  unit fall back to rest at once (claim 16) without retaliating and without a scatter request from
  the damage (claim 19 case 2 and S-0208 claim 15 need the scatter property once retaliation is
  ruled out); the subjects are of types that are not fraidycats (Terms); the area-guard threshold is 0, so that the
  human-controlled house's armed vehicles otherwise rest in *area guard* (claim 15 row 12). The
  damage comes from a second unit of the human-controlled house given a force-fire order on the
  subject, with a weapon too weak to kill it in one shot. The recorder reads the subject's slots
  on the tick of the damage and on the next tick. A run in which the subject processes on the tick
  of the damage, dies, or has its destination changed on that tick by anything other than the rows
  under test, is set aside. In (a1) and (a2), a run in which the unit holds no target on the tick
  of the damage is invalid.
  - (a) Infantry (claim 14). (a1) An armed infantry unit of a type that does not default to area
    guard, in *guard*, holding as target an unarmed enemy building in its weapon range (taken by its
    own per-tick search): expected *attack* queued on the tick of the damage (row 2), current by
    the next tick unless the unit is not ready to change. (a2) The same unit in *area guard* on its
    own cell, holding the same kind of target: expected nothing queued and *area guard* still
    current (the exception for *area guard*). (a3) An armed infantry unit whose type the test file
    marks as defaulting to area guard, in *guard* with no target and no object of another house
    within its sight: expected nothing queued (row 5; without it, row 11 would queue *area
    guard*). Falsified by *attack* neither queued nor current by the next tick in (a1), by *attack*
    queued in (a2), or by *area
    guard* queued in (a3).
  - (b) Vehicles (claim 15). (b1) An armed vehicle that is not a harvester, given an area-guard
    order on a cell 10 cells away, with no object of another house within its sight, is damaged
    while it drives there with that cell as destination: expected nothing queued (the exception for
    *area guard*; without it, row 3, which comes before row 8, would queue *move*).
    (b2) An armed vehicle that is not a harvester, in *guard* with no target and no object of
    another house within its sight: expected nothing queued (row 8; without it, row 12 would queue
    *area guard*). Falsified by *move* queued in (b1) or *area guard* queued in (b2).
  - (c) *Eaten* (claims 15 and 28). A test map places an unarmed vehicle that is not a harvester, a
    loaner or a deployer, of the human-controlled house, in *eaten* on open ground, with no target
    and no destination; a second one is placed in *capture* the same way. Expected: at each
    processing the vehicle in *eaten* falls back to rest and nothing is queued (the exception for
    *eaten*), so *eaten* stays current over 10 processings; the vehicle in *capture* queues
    *guard* at its first processing (row 7; the hunt rule of claim 28 does not apply to a
    human-controlled owner). Falsified by *guard* queued for the vehicle in *eaten*, or by nothing
    queued for the vehicle in *capture*.
  - (d) Bunker variant of row 12. The test file of cases a and b. An armed vehicle of the
    human-controlled house that is not a harvester is ordered into an empty bunker of its own
    house and then ordered to attack an enemy unit within its weapon range, which a map trigger
    destroys a few ticks later. Expected: at its first *attack* processing after the loss of the
    target it queues *guard*. Control: the same vehicle standing on open ground: it queues *area
    guard*. A run in which the recorder shows the vehicle out of contact with the bunker, or with
    a destination, at that processing is invalid. Falsified by *area guard* queued in the bunker,
    or by *guard* queued in the control.
  - Runs: three per case.

- **B30 (F1 and F9: products of 2⁶³ or more).** (a) Named test file: rate of *guard*
  20000000000000000 game minutes (held as a single near 2.0 × 10¹⁶; times 900 about 1.8 × 10¹⁹,
  above 2⁶³, so *D*<sub>r</sub> = 0). An armed vehicle of the human-controlled house rests in
  *guard* with no object of another house within its sight and its weapon not rearming. Expected:
  processing intervals of 1 or 2 ticks (the standard delay 0, 1 or 2, where a delay of 1 or less
  means the next tick), 1 about twice as often as 2. A second run with the rate written as
  2000000000000000000% (a double-precision product, F1) gives the same. Falsified by any interval
  above 2, in particular by no further processing at all (a conversion that saturates at
  2³¹ − 1). (b) As scenario B28, case f, with the gate's deploy time 20000000000000000 game
  minutes instead. Expected: the door is at rest open at most 2 ticks after the gate's behaviour
  becomes *open*, and the unit then passes. Falsified by a door that is still moving, or still
  closed, more than 2 ticks after *open* becomes current. Runs: 100 intervals per run in (a),
  three in (b).
- **B31 (claim 26: row 1 and the slave master that wakes its slaves anyway).** Setup and recorder
  of S-0402 scenario H7 (a slave miner of the human-controlled house; the recorder logs the tick at
  which each slave leaves the miner), with this scenario's own named test file setting the slave
  count to 3, the kick delay to 330 ticks and the slave-miner short scan to 7 cells. These are
  invented test values, set whatever the stock data gives. The miner stands on an ore cell (as
  in H7, variant c, so that the miner's cell meets the wake-up rule), and the same named test file
  gives the slave miner's type no harvester flag (so that row 4 of claim 26 does not apply) and *area guard* a rate of 0.05
  (*D*<sub>r</sub> = 45). At tick 100 the miner is given the guard key command, which gives a
  vehicle that controls slave units *area guard* on its own cell, armed or not (S-0213, claim 18,
  table T2); the recorder logs the start tick *s*<sub>m</sub> of *area guard* and every processing
  tick of the miner with its delay (the processing timer's next expiry minus the tick), and the
  land type of the miner's cell at each processing (the slaves may exhaust that cell).
  Observable: the ticks at which slaves leave, and the delay of each processing of the miner.
  Expected: (a) the slaves are woken anyway at the first processing of *area guard*, although the
  kick sum *s*<sub>m</sub> + 330 has not passed: the first slave leaves within 100 ticks of that
  processing, long before tick *s*<sub>m</sub> + 331, and that processing's delay is 46 to 50
  (45 + *U*(1, 5), no row ended it). (b) Each later processing at which the slave group is at rest
  again, the tick is above *s*<sub>m</sub> + 330 and the miner's cell is still ore is a row 1
  processing: the slaves leave again, and its delay is 45, 46 or 47 (the standard delay). A
  waking processing at which the cell is no longer ore counts with (a) instead, since row 1 may
  then fail on the group's rest tick (S-0402, claim 14): its delay is 45, 46 or 47 if row 1 still
  holds and 46 to 50 otherwise, and it tests nothing here. Over all runs, 45 is seen among
  those delays and 48 to 50 never. Falsified by no slave leaving before tick *s*<sub>m</sub> + 331
  (no wake anyway), by a delay of 48 to 50 at a processing of (b) (row 1 not ending the
  processing), or by a delay of 45 at the first processing of (a) (a row-1 outcome without the
  kick sum having passed). Runs: five, each of 6000 ticks.
- **B32 (claim 37, F10: the approach step).** Common setup: a tiny flat map at height 0 with open
  clear ground, game speed index 2, one human-controlled house and one bystander computer player
  kept outside the observed area. A named test file sets the rate of *attack* to 0.07
  (*D*<sub>r</sub> = 63), the approach reset multiplier to 1, and gives one ordinary ground vehicle
  type and one ordinary infantry type a weapon of range 5 cells (1280 leptons, so *r*′ = 1152) with
  no minimum range and no close-range property. These are invented test values, set whatever the
  stock data gives. The recorder logs, each tick, the attacker's behaviour, target, destination
  (whether a cell or an object, and which), position, and processing ticks; and the target's
  position.
  (a) An unarmed enemy vehicle stands still 10 cells from the attacker; the attacker gets an attack
  order. Then the same with the target 3 cells away, and again 3 cells away with the attacker's
  weapon rearming at the order. Expected: in the first run the first destination appears on an
  attack processing tick and changes only on processing ticks; in the other two runs the attacker
  never gets a destination and does not move. Falsified by a destination set on a tick that is not
  a processing tick of *attack* (other than by the order itself), or by any destination in the
  in-range runs. Add a run with the enemy 3 cells away but a wall blocking the weapon's
  line of fire: use a projectile subject to walls and a warhead that cannot damage walls, and
  keep an open firing position available around the wall. Expected: an approach destination
  appears at an attack processing even though geometric distance is within range (S-0307
  claim 11 refuses the line of fire). Falsified by staying without a destination and target
  loss in that run while an eligible firing position exists.
  (b) The target is an enemy vehicle under a scripted move order that drives it straight away from
  the attacker at a steady speed, starting 7 cells away. Expected: after the first destination,
  each new destination appears on the first attack processing tick at which the 3-D distance from
  the target's reference point to the held destination's reference point is above 1152 leptons,
  never earlier. A second run gives the attacker's type the can-recalculate property *no*:
  the first destination is then never replaced while it is held. A third run stops the target 2
  cells beyond the destination: the attacker's destination is not cleared at any attack processing
  while it is held, even after the range test passes (when the attacker stops belongs to the
  movement specs). Renewals are expected only while the target is out of range of the attacker.
  Falsified by a renewal at or below the threshold, a renewal on a tick that is not a processing
  tick, no renewal at the first processing above the threshold while the target is out of range,
  any replacement of a held destination with the can-recalculate property *no*, or a destination
  cleared by an attack processing because the target came in range.
  (c) As (a), first run, with the attacker's type given the can-approach property *no*. Expected:
  the attacker still approaches (the *attack* exception of claim 37 d). Then the guard key command
  (S-0213 claim 18) puts it in *area guard* and an enemy vehicle drives into its sight but out of
  its range: it approaches as well (human-controlled owner), and so does the same attacker with
  the property *yes*. Falsified by an attacker that keeps its target without moving, or drops it.
  (c2) *Hunt* with a human-controlled owner. The named test file also gives the attacker's type a
  sight of 10 cells (an invented test value) and no stupid-hunt flag. A test map places the attacker of the
  human-controlled house in *hunt*, with an unarmed enemy vehicle standing 8 cells away. In a first
  run the attacker's type has the can-approach property *yes*: expected, it gets a destination on
  a *hunt* processing tick and approaches. In a second run the property is *no*: expected, it
  never moves and never holds a destination, and it holds no target at the end of any *hunt*
  processing tick. Falsified by no approach in the first run, or by movement, a destination or a
  target kept to the end of a processing tick in the second. A pair of runs in which the first
  run shows no approach is invalid for the second (the search did not find the enemy).
  (d) The same named test file also gives *attack* a rate of 2 game minutes (*D*<sub>r</sub> =
  1800), so that attack processings are far apart. The attacker vehicle stands on a shore; the
  target is an enemy hover vehicle on the water 3 cells away, in range, and the water beyond it is
  wider than 10 cells. The attacker gets an attack order; after its first processing the target
  drives straight out over the water under a scripted move order. Expected: the attacker drops
  its target within one tick of the first tick on which the target is out of range (the target's
  water cell is never in the attacker's reach region), not at an attack processing tick.
  Falsified by a target held until the next attack processing tick.
  (e) An infantry unit of a type that deploys and fires when deployed, deployed, gets an attack
  order on an enemy 10 cells away. Expected: it neither moves nor gets a destination while
  deployed. Falsified by a destination or movement while its sequence is a deploy kind. A run in
  which the order does not leave *attack* current with the enemy as target is invalid.
  (f) Repeat (a)'s distant-target run with each ordinary attacker type's speed set to zero in
  the named test file, keeping its locomotion class and movement zone unchanged. Issue the same
  attack order as in (a): zero type speed alone excludes neither *movable* nor *able to fire*
  under S-0214's Terms. Expected: the first attack processing with a target
  gives the same first destination as the positive-speed control, with no displacement for the
  ordinary driving or walking locomotor. Falsified by exclusion from the step solely because
  speed is zero, or a different destination with identical positions, tick and search conditions.
  A run is void if the target is not assigned or *attack* is not current at the processing.
  (g) Q-0161: same-tick acquisition and approach. Use the shore and unreachable enemy hover
  vehicle of (d), stationary 8 cells away. A named test file gives the attacker a guard
  range of 10 cells, weapon range of 5 cells, the can-approach property *no*, and a long
  *guard* processing delay. Put the attacker in *guard* with no target or destination.
  Record the processing and targeting timers, target and destination every tick, plus the
  scenario random state, with the read-only recorder. Make a control run with the enemy
  on reachable ground at the same distance: its acquisition must be confirmed there, or
  the pair is void. In the shore run, record search ticks both with and without a
  behaviour processing, repeating with different recorded seeds until both occur.
  Expected: on either kind of search tick the targeting timer restarts, but the unreachable
  target is absent by the end of that tick and no destination is held. The reachable
  control keeps the acquired target. This distinguishes approach after search from approach
  before it, independently of the processing timer. Falsified by the unreachable target
  surviving its acquisition tick in the stated conditions. As a further control, issue an
  *attack* order against a reachable distant target: a destination supplied by processing
  remains held and does not trigger this later no-destination rule. For the unchanged
  in-range case, repeat (d) with the unreachable target stationary in range across a known
  attack processing tick: expected target kept and no destination. These end states
  test the outcome only; no internal evaluation count is required by claim 37 a2.
  (h) Q-2171 question 4: the reach comparison at a bridge (claim 37 a1). A named map has one
  bridge only: an enabled high bridge over water, of at least 10 deck cells, whose two banks
  are also joined by land away from the bridge, so that both lie in one reach region of the
  ground movement zone.
  The bridge stays intact throughout, and each named object on or under it stands within 1 cell
  of the bridge's centre line, so that each deck read finds the bridge's link (S-0212 claim 4).
  Use case g's named test file, attacker vehicle and recorder, with the attacker in *guard*,
  no target or destination, and an unarmed enemy object stationary 8 cells away. Run 1: the
  attacker on the deck, the target an enemy vehicle on the same deck. Run 2: the attacker on a
  bank cell, the same target. Run 3: the attacker on a bank cell, the target an enemy hover
  vehicle on the water under the deck. Run 4 (control): the attacker on the deck, the target an
  enemy hover vehicle on open water beside the bridge, with no bridge over its cell. Each run
  has a control run, identical except that the named test file raises the attacker's weapon
  range to 10 cells, so that the target is within range and the per-tick rule keeps it under
  every reading. A run is void unless its control's target is held at the end of the control's
  acquisition tick: that shows the search takes this target from this cell. Expected:
  in runs 1 to 3 the target is still held at the end of its acquisition tick, with no
  destination; in run 4 it is absent by the end of that tick, as in case g. Reading both cells
  for their ground predicts a drop in runs 2 and 3 and a kept target in run 4. Reading the
  actor's cell for its ground predicts a drop in run 1, and reading the target's cell for its
  deck only when the target is on the deck predicts a drop in run 3. Falsified, in a run that
  is not void, by a drop in runs 1 to 3, or by a target kept to the end of its acquisition tick
  in run 4. Inconclusive: a
  run with no recorded search tick that restarts the targeting timer while the target is within
  guard range; a run in which the bridge is damaged or a named object stands farther than 1
  cell from the centre line; a run in which a named object is not recorded in its stated layer
  (the attacker on the deck in runs 1 and 4, the target on the deck in runs 1 and 2, and on the
  water under the deck in run 3).
  Record the stated actor and random-stream observables each tick, using the measurement
  setup's scripted inputs. No external controller may play either house or alter game state.
  Run without mods or engine add-ons, apart from the named test data permitted by the common
  setup. Runs: three valid search ticks of each kind in case g, and three per other case and
  attacker kind (vehicle and infantry in a, b, c, c2 and f; vehicle only in d, g and
  h; infantry only in e).

- **B33 (claim 13, Q-0202).** Common setup and fixed game speed 2. First repeat B15 b/c on a
  tiny open map with three ordinary move route points, no held attack-move input modifiers and
  no enemies near the route. Record destination, current/queued behaviour and issued/applied
  orders around each arrival fall-back. Expected: a next-point order is issued, but does not
  supply a destination until its ordinary order boundary. Immediate replacement during the
  fall-back falsifies the route distinction. Three runs per actor kind.

  The following are conditional checks, not reproducible setup claims. Ordinary attack-move
  input clears the guard spot and ends a route, so do not write state to manufacture a collision.
  If a recorded stock input sequence produces one, preserve that sequence as the setup, then
  replay it three times under the common setup. Record state immediately before and after the
  fall-back, distinguishing memory's destination from the actor's destination.
  - Guard spot and resumable memory: expect the guard spot as final destination, with current
    and queued behaviours exactly as claim 13's readiness cases state; memory remains and
    engagement ends. A final remembered destination instead falsifies the priority.
  - Vehicle approach entries and resumable memory: record the list before refresh and after
    any destination assignment. Expect the approach row to use only the remaining list, never
    a saved pre-refresh entry. Target-only resumption can retain an entry; if consumed it
    becomes destination without replacing the resumed *attack* slot state. The contrary
    falsifies the list/slot distinction.
  - Route and resumable memory: expect the refreshed state at fall-back completion, unless
    the guard-spot or surviving approach row changes it; the route's generated order acts at
    its later boundary. Immediate route destination replacement falsifies this result.
  - Repeat any available guard-spot/route case with non-engaged memory outside *guard*:
    expect removal, the same applicable row effects as a no-memory control, and ordinary
    rest-table outcome. Suppression solely because memory existed falsifies claim 13.
  These collision cases remain `unmeasured` until a legal input sequence establishes the
  prerequisites; this scenario proposes recording them, not a state-writing harness.

- **B35 (claims 1 and 11).** Use the common skirmish setup, speed index 2, with no external
  controller playing either house or altering game state, and no mods or engine add-ons
  except the named test data below. Use a tiny open-ground map and one unarmed map-placed
  building of the neutral house with no turret. Record its body
  state and current and queued behaviours at entry and each turn. Expected at game set-up:
  idle body and *guard* queued, not a construction body or *construction* queued. One run.
  Named test art gives its idle body a range of 3 stages and delay 4 ticks; a second run gives
  the range delay 0. Observe the ready mark through a read-only recorder before each per-tick
  start. Expected: landing on the last stage sets it in the first run, wrapping alone does not.
  Every non-advancing turn sets it in the zero-delay run. Repeat with a named test type that
  has a turret: idle *guard* does not set it through this body-range rule. No weapon or other
  ready-mark-setting rule is present. Replay S-0406 PL11 with turreted and non-turreted types
  to confirm that queued *construction* qualifies in both. A different entry body/queue or
  any ready-mark change contrary to these conditions falsifies the corresponding rule.
  Runs: one per case. The observation area is the test building and its surrounding cells.
  The common bystander void rule applies. Remove the named test data afterwards.

- **B34 (claim 26, Q-0231).** Tiny flat skirmish test map, game speed index 2, no external
  controller playing either house or altering game state, no mods or engine add-ons,
  one human-controlled house and one bystander computer player kept away from the
  observed area. Place a stock armed vehicle, then in a separate run a stock armed infantry
  unit that is not a deployer, in *area guard* with no guard spot. Use the read-only recorder
  to identify its first processing and verify at entry: no target, no destination, nothing
  queued, targeting timer expired and passive acquisition allowed. A run failing those
  conditions is void; do not write state to manufacture them. Repeat with no enemy in search
  range and with one eligible stock enemy as a named test object inside the area search range.
  Record guard spot, target, both behaviour and targeting timers, infantry idle timer and
  sequence, and scenario random-stream state
  on that processing tick and the next processing. Three valid placements per kind and enemy
  condition, 800 × 600; scripted input may keep other objects out of the observed area.
  Expected: the spot is its own cell and the targeting timer restarts on the first processing
  by claim 33's area-delay formula and timer draw. With the eligible enemy selected, the target
  is acquired on that processing and the next processing is 1 tick later, with no delay draw.
  With no enemy, the actor attempts its idle action and chooses claim 26's ordinary delay.
  The timer draw has claim 6's earlier stream assignment than the delay draw. For infantry,
  additionally require the idle timer expired, no movement, not prone, not in the attack
  state, and a sequence eligible under S-0107 claim 14. Expect the idle timer to restart on
  that first processing by that claim; a visible idle sequence is not required on every run. The idle
  action's draws and eligibility remain owned by S-0107, not inferred from a total draw count.
  Falsifiers: a guard spot other than the actor's own cell, a targeting timer left expired
  until the next processing despite the verified conditions, acquisition deferred despite
  the first search selecting the enemy, a processing interval other than 1 tick after
  successful acquisition or outside claim 26's ordinary delay in the no-enemy run, a delay draw
  after successful acquisition, or no idle-action attempt in the eligible no-enemy infantry
  run. Per-tick searches or actions of other objects must be separated by the recorder; if
  they cannot be separated, that run cannot establish the draw assignment.

- **B36 (claims 1, 32 and 33, Q-0241).** Common tiny skirmish map, speed index 2, 800 × 600, no external
  controller playing either house or altering game state, and no mods or engine add-ons
  except the named test data below. The map places a stock turreted defence with a weapon for
  a non-player owner, and a named enemy unit in its range; keep the bystander away. Use the
  read-only recorder to record current, queued and suspended behaviours, ready mark, body
  state, target, processing timer and first shot. Capture them at the end of set-up, before
  the first turn, then at both per-tick-start boundaries and at the end of each tick through
  tick 1000. Repeat with a named test type identical except unarmed, with a non-turreted
  armed type, and with a non-turreted base that gets a turret-bearing upgrade installed by
  the map. Give that base no weapons of its own and the first upgrade an ordinary primary
  with a burst count of 1 and no secondary; this is the failed-gate fixture. The same-weapon
  comparison below changes only the upgrade's secondary. These are named test settings,
  not assumed stock values. For the upgrade case, also capture the host just before and
  just after the upgrade is installed. Three runs per case; remove named data afterwards. Keep the named
  enemy unable to damage the buildings.

  Expected, in every case: at the end of set-up the building has the idle body, current
  *none*, *guard* queued and the ready mark set. Installing the upgrade changes neither the
  host's slots nor its mark. At the building's first per-tick start *guard* becomes current
  and the mark is cleared, and *guard* is processed in that same turn. The armed defence
  then searches under claim 33 and starts *attack* at a later guard processing. Without
  upgrades it fires when eligible. An upgrade-only defence fires only when claim 32's
  position-1 weapon gate passes; failure leaves it targeting and attacking without shots.
  The unarmed one does not fire. Add a no-enemy control and a second upgrade-only case
  differing solely by making the secondary weapon the same type as the primary: that case
  must make both firing attempts in the first eligible attack processing.
  Falsifiers: a clear ready mark at the end of set-up; *guard* still queued after the
  building's first per-tick start; a first processing of *guard* later than the first
  turn; or any of these differing between the armed, unarmed, non-turreted and upgrade
  cases. *Guard* already current at the end of set-up falsifies only the order within
  placement (the grand opening after the queue), not the first-turn result.
  A human-owner set-up comparison remains unmeasured: skirmish drops map building entries
  for the human-controlled owner (S-0009 claim 27), and constructing or capturing a
  building is not set-up placement. Do not substitute either for that comparison. Do not
  inject a ready mark or force a start. Enemy action before the first recorded turn
  invalidates the placement observation.

  Also record firing attempts and firing-check answers in the two upgrade cases.
  A failed gate producing either attempt, or a passing same-weapon gate missing either
  attempt, falsifies claim 32. Record the ready mark and processing timer at attack entry
  and exit: a failed gate must still set the mark, retain target and *attack*, and choose
  1 tick without a delay draw. A contrary state or delay falsifies that clause.

  **Extensions (proposed, not completed measurements).** Repeat three runs per named
  map-local variant, recording the natural seed, speed index 2 and 800 × 600, with
  firing-check answers, selected weapons, attempts, shots and target health recorded.
  Do not fix the seed or write random-stream state. Use different primary and secondary weapon types
  with only the primary burst count above 1, then only the secondary burst count above 1,
  then both counts at 1. Expect paired attempts in the first two and suppression in the
  third. Reverse the types in occupied upgrade positions 1 and 2, only one of which
  passes the gate: position 1 must decide suppression regardless of which supplies
  the selected weapon.
  A contrary outcome falsifies the corresponding gate clause. Add a first upgrade with
  no primary, a secondary, and a host primary weapon: expect suppression even if the
  secondary burst count exceeds 1. For the ordinary-versus-elite predicate, give the first
  upgrade distinct ordinary weapons with burst counts of 1, but identical elite weapons,
  then test a rookie and an elite host: both must remain suppressed. Conversely give it
  identical ordinary weapons and distinct elite weapons: both hosts must make the paired
  attempts. An elite-only change affecting permission falsifies the ordinary-weapon rule.
  With distinct weapons in a passing case, record attempt order and rearm state: the
  primary attempt precedes the secondary, whose attempt is not prevented by rearm begun
  by the primary. A reversed or missing attempt falsifies this ordering clause.
  Repeat the suppression and passing cases with upgrades installed during play rather
  than by the map: expect the same permission and attempt outcomes. Record a no-upgrade
  control's ordinary selected-weapon attempt, not a forced pair. A placement-dependent
  gate or an upgrade-style pair in that control falsifies the corresponding clause.

  Finally use a primary burst count above 1 with no resolved secondary anywhere, recording
  attempts, projectiles, burst count, rearm, ammunition and random state at each attempt.
  The primary attempt occurs; the empty-secondary attempt makes no change and produces no
  firing-check answer (S-0301 claim 9). Any secondary shot or state/draw change from that
  attempt falsifies the rule. Repeat with the host's animation-delayed-fire flag enabled
  and a positive named-test delay: the passing pair is still immediate and starts no
  delayed shot. A failing original gate starts neither attempts nor delay. A no-upgrade
  control instead starts the ordinary delayed shot. Use S-0301 F17's recording and
  falsifiers, three valid runs per case. Test skipped positions and sale/reinstallation
  separately under S-0406 PL16: position 1 alone gates, while provider eligibility follows
  the count. For a level-skip control, use PL16 part a's fresh host and install a level-2
  upgrade whose weapons would fail the gate if it occupied position 1. Verify a nonzero
  count, empty position 1 and occupied position 2. Expect the ordinary selected-weapon
  path, not suppression or an upgrade-style pair. Repeat with animation delay enabled:
  the ordinary delayed shot starts. Suppression by position 2 or an immediate pair
  falsifies the bypass rule. Repeat the unarmed case with a separate mark
  reading immediately at guard entry and exit, before any other readiness rule can run:
  claim 32 predicts no mark set by unarmed guard itself; a clear-to-set transition inside
  that processing falsifies it. If the mark was already set at entry, that sample cannot
  isolate a setter. Do not clear it by writing state. Keep both enemy and building alive
  and stationary through tick 1200.
  No controller may force behaviours or firing-check results. Test the quirk's off state
  in our engine: a failed gate must allow an ordinary selected-weapon attempt instead.

- **B37 (claim 38, Q-0238 and Q-0312).** Reuse S-0212 P26's unreachable-destination
  skirmish setup for the neutral and special houses, speed index 2, 800 × 600, with no
  external controller playing either house or altering game state, and no mods or engine
  add-ons except the named test-map data below. Use the read-only recorder, not writes, to record the house's base centre,
  base-start cell, radius, selected centre, final destination and scenario random-stream
  state at give-up. Separate unrelated draws from other objects. Three valid runs per
  owner and case. Cases: no base/start; one and then two owned map buildings of unequal
  purchase prices; nonlethal damage to one building without entry, removal or owner
  change; loss or owner change of one of them; loss of the last building. Keep the units in the same area while moving
  the buildings between runs, and move the failing unit between two unreachable areas
  while keeping the house records fixed. Use a flat map first, then a plateau at the
  local area's top edge, then a smaller local area shifted within the map.

  Expected: initial empty records/zero radius without a start or building; base-record
  updates exactly as F12, including unweighted distance sum divided by price-weight
  sum and no refresh on damage alone. Compute the selected centre from the recorded
  radius/direction values by F11, including snapping, whole-point rejection outside
  the coordinate bounds, lateral correction and the first usable diagonal cell.
  The no-base result uses zero world origin and still consumes the ranged radius
  assignment followed by one raw direction value. Moving only the failing unit cannot
  alter that centre for identical house records and draw results, although its reach
  region can alter the final nearby-search result. Compare that result separately
  against S-0212 T5, not against the centre itself. Falsifiers include a different
  centre from F11, a radius/record change contrary to F12, a unit-relative no-base
  point, an outer-choice index draw, omitted direction draw, or a correction/final-search
  draw. Record the seed; repeat naturally different seeds rather than writing the
  stream. A start-assignment case or separately set centre case may be checked only
  after a repeatable ordinary setup or map-action input for that house is established
  and recorded. Neither is supplied by this fixture, so record those branches untested,
  not as confirmed by the no-base and owned-building runs. A snapped empty world point would
  falsify the no-retry qualification.
  Named test-map rules may give unequal invented prices and keep buildings alive
  long enough to record refresh. Remove the named test data afterwards.

- **B38 (claim 11a).** Extend S-0402 H20's human-owned, non-contact ordinary
  harvester on a docking cell, with the same stock types, speed 2, remote bystander,
  no external controller playing either house or altering game state, and no mods or
  engine add-ons. Use three runs per case. Give a harvest order and record
  the selected field cell, failed route category, destination, current and queued
  behaviours, readiness, processing timer and each processing tick with a read-only
  recorder and scripted input. Use S-0212 claim 23's fixtures to distinguish a
  gate refusal, a failed search within 1 cell that does not give up, and a failed
  search to a field more than 1 cell away that gives up. Keep a refinery owned
  throughout, and separate any new order from the failure reaction.

  Expected: only the give-up case queues *guard*. It does not force a start or
  restart the processing timer at queueing. At the next permitted per-tick start,
  *guard* becomes current even if the old harvest delay or route retry delay has not
  expired. A first start processes it that turn; a late start first processes it on
  the next update. Record both boundaries separately. The failure-specific late-start
  comparison remains conditional under claim 11a's backing, not an established fixture. In the other cases, if neither an order nor
  another rule changes the behaviour, *harvest* remains current and its next
  processing follows S-0402 claim 3's interval and seeking rule. Record whether
  failure preceded or followed the turn's per-tick start rather than assuming
  an end-of-tick queue is still pending. A docking-cell-only guard force, a start
  held back solely by either timer, or a harvest processing after guard has started
  without a new harvest start falsifies the claim. Reuse S-0212 P26 and B37 for
  neutral/special failure-tick destination assignment and its separate draw audit.

- **B40 (claim 13, repair/reload route actions).** Use the common skirmish setup and recorder
  with route-mode scripted input on a tiny open map. Place an own-house repair building and an
  allied bystander's repair building, and commit a vehicle's route with either building as its
  next point after an ordinary move point. Run damaged and full-health vehicle variants.
  Separately use a reload building and an actor with finite ammunition, below and at its
  imported capacity. A named test map supplies a reload building and finite ammunition for a
  ground vehicle if stock types cannot express that comparison; no service outcome beyond
  order selection is assumed. Record the classified click action, route-generated order,
  destination and behaviour at the subsequent order boundary. Expected: when the ordinary
  classification permits an object action other than *select* or *toggle select*, the damaged
  non-infantry repair case and the below-capacity reload case issue that object action.
  Full-health repair-only and full-ammunition reload-only controls use the cell fallback.
  Add infantry at a repair-only building and an unlimited-ammunition actor at a reload-only
  building: neither service condition qualifies. At a building with both service flags,
  either qualifying condition suffices. Falsified by an object order in a nonqualifying
  control or a cell fallback when all service and action guards hold, or by destination
  assignment inside the fall-back instead of later order application. Three runs per case.
  The original's local-player alliance dependence and asymmetric-alliance guards remain
  unmeasured under claim 13's qualifications; these mutually allied fixtures do not test them.
  For our engine's design departure, additionally check a skirmish with a person and a
  non-allied AI house, and an otherwise legal route issued by that AI to its own repair
  building for a damaged vehicle. Record the classified action and issued order, holding
  the fixture free of unrelated orders. With *enter* classification, expect the object order,
  regardless of which house is being viewed; a cell fallback caused solely by the viewed
  house's alliance falsifies the departure. Repeat with below-capacity reload and with no
  person playing. These are engine acceptance cases, not proposed original-game measurements.

- **B41 (claim 33a, conditional allied-target investigation).** Use the common tiny-map
  skirmish setup at game speed 2, stock data, no external automated player or mods,
  and a read-only recorder
  of target, owner, behaviour, processing timer and berserk state. First establish that a
  computer-owned repair vehicle carrying an engineer can acquire a damaged own-house vehicle
  as a healing target. Record acquisition and every subsequent own update, including ticks
  when behaviour processing is not due. Compare a human-owned equivalent. Expected, if the
  nonhuman vehicle reaches target maintenance holding that target: the target is cleared
  before subsequent behaviour processing, regardless of either timer, while the human-owner
  rule alone does not clear it. A nonhuman target retained through that point with none of
  claim 33a's exceptions would falsify the rule. Failure to establish the held target does
  not test it. Run three repeats after validating the fixture. For the berserk exception use
  S-0610 P4's neutral-house pair: while berserk it can retain its allied target; expiry's
  separate target clearing is not evidence of this rule. The engineer, occupation, charging
  and controlled-transport comparisons remain unmeasured until fixtures isolate their
  targets at the same maintenance point; no state injection is proposed.

- **B42 (claims 5 and 10: bridge fall processing and centre-cell revert).** Reuse
  S-0511 scenario B3's turn-order variant and B10's target-loss variant, including their
  setup, recorder and run counts. Record due processing and order changes during a deck
  drop, with a protected landing survivor as the same-turn control. Expected: due
  processing continues after the fall step, ordinary locomotion and firing remain refused
  while falling, and a surviving landing removes those refusals that turn. For target loss,
  compare an exact-centre-cell attacker with a neighbouring-cell attacker and a unit merely
  moving toward the centre cell. Include a suspended override with saved target and
  destination, and a control without one. Expected: only the exact-cell attack target
  triggers the revert, saved state returns, and a restored target equal to that cell is
  then cleared. No attack-move memory refresh occurs. Suppressed due processing,
  a blanket destination clear, loss of the neighbouring target or a missing revert
  falsifies the corresponding claim.

- **B43 (claims 22 and 29, Q-0797).** Reuse the common skirmish setup, speed index 2,
  800 × 600, stock data, no external automated player or state-changing add-on, and
  no mods. Use S-0207 D30's driving and ship fixtures for an ordinary same-cell move
  and an ordinary move ending with a final track, and S-0314 H19 f's accepted
  same-cell depot-entry fixture. Keep contact accepted and approach destinations empty
  for the entry observation. Record both destinations, track presence, current and queued
  behaviour, processing timer and the state immediately before and after each due
  processing and locomotion update. Three valid runs per case, retaining the common
  bystander void rules. Do not force a timer, behaviour, contact or destination.

  Expected: due ordinary *move* processing retains a present cell goal even at cell
  equality. Locomotion removes that goal under S-0207 claim 26, after completion at
  the next qualifying no-track update, or at the first such update after a same-cell
  order. The next due *move* processing with no destination, no movement and no queued
  behaviour falls back and chooses 1 tick; it does not have a new fixed cleanup timer.
  The accepted depot-entry processing retains its building goal at cell equality,
  including a due processing after a same-cell entry. An arrival exchange can queue
  *sleep* without deleting that goal. Record any guidance, refusal, replacement or
  servicing event separately, not as equality cleanup. Deletion inside an otherwise
  ordinary move/accepted-entry processing solely because cells match, a cleanup-induced
  processing-timer reset, or an automatic one-tick entry-goal deletion falsifies the
  corresponding clause. If entry changes phase before a due processing can be captured,
  that run cannot test processing retention; repeat the same-cell fixture rather than
  inferring a result from a serviced vehicle.

- **B44 (claims 4 and 11: fresh paratrooper readiness).** Reuse S-0522 claim 17's
  fresh human-owned payload fixture and P10, with the common skirmish setup, speed
  index 2, 800 × 600, stock data, no external automated player or state-changing
  add-on and no mods. First use an isolated clear landing area without an enemy.
  Record falling, current and queued behaviours, readiness, processing timer, target
  and destination from release until 60 ticks after landing. Include readings around
  each due processing and start. Three valid runs, with the common bystander void rules.

  Expected: release leaves current *none*, ordinary rest queued, and no target or destination. While
  falling the unit is not ready and rest stays queued. A due processing with *none* current chooses
  exactly 450 ticks without a delay draw or changing those slots. Record random state around that
  processing separately from release and animation draws. Landing itself issues no order and
  replaces neither slot; the resting behaviour starts when ordinary readiness permits, resets the
  timer, and is processed under claims 3 and 11, without waiting for the 450-tick delay to expire.
  Do not infer this start from the first shot or require it on the landing tick. Repeat with P10's
  eligible enemy and named type override selecting *area guard*, retaining its setup and falsifiers.
  Starting queued rest while still falling, a non-450-tick *none* processing, a delay draw for it,
  or a landing-generated replacement order falsifies the corresponding clause. Variant b, run
  separately: a scripted move click on a distinct free cell during descent, with S-0522 scenario P10
  (c)'s expectations: the move order kept through the fall and carried out after landing. In variant
  b rest is not expected to stay queued after the click. A move started while still falling, or a
  click that gives no order, falsifies it. Do not force slots, timers or readiness. Other readiness
  conditions can still delay the start after falling ends.

- **B45 (additional discriminating fixtures for claims 6, 10, 11 and 32).** These are
  proposed measurements, not results. Use the common tiny-map skirmish setup, game speed
  index 2, 800 × 600, no external automated player or state-changing add-on, and no mods
  except each part's named test-map rules/art. Read state only; do not force timers,
  targets, firing-check answers or behaviour slots. Keep subjects alive with named
  high-health and low-damage test weapons. The common observed areas and void rules apply.

  - **(a) Targetless vehicle hunt and draw assignment.** Place an ordinary armed driving
    vehicle of the human-controlled house in *hunt*, with no eligible enemy anywhere
    within search range. Name test-map rules setting the hunt rate to 0.07 game minutes
    and clearing the stupid-hunt, hover-attack and deploy flags. It is not a slave master,
    harvester or distributed-fire type. Record each search, targeting-timer restart,
    processing expiry and full scenario-stream state immediately around the processing,
    excluding other actors' draws rather than estimating them statistically. No target
    means no approach, and this vehicle has no infantry idle action. For each observed
    entry stream state predict both assignments by S-0202: timer *U*(0, 2) then delay
    *U*(0, 2), or the reverse. Expected: targeting delay *N* plus the first value,
    processing delay 63 plus the second. A different value, unexpected extra draw,
    acquisition or movement falsifies the applicable rule; unrelated draws make a sample
    inconclusive, not confirming evidence. Collect 30 samples whose two predicted timer
    and delay pairs differ, across naturally seeded runs. This part does not test
    ordering after infantry idle, approach or scatter draws.

  - **(b) Revert at enemy cloak start.** Extend B7 b with an enemy vehicle given a
    self-cloaking type flag by named test-map data. Its weapon causes a nonlethal hit
    and an override on the observer; then a test-map script stops the attacker so its
    ordinary cloak conditions can become satisfied. Use a long observer processing
    delay and a weak observer weapon so the attacker survives until cloak starts.
    Give the observer a named-test weapon rearm delay longer than the attacker's cloak
    delay and fade duration combined, so its response shot cannot keep restarting the
    attacker's cloak delay. Record all hits and require a hit-free interval long enough
    for ordinary cloak eligibility. Repeat with no sensor, then with a sensor of the
    observer's own house covering the attacker's cell before the hit, outside
    adjacency-uncloak range. Use S-0516
    claims 4–6 and 21–23 for cloak eligibility and coverage. Record cloak transition,
    sensor coverage, target and all behaviour slots immediately around cloak start.
    Expected without coverage: suspended behaviour restored and suspended slot empty
    at that event. Expected with coverage: no cloak-caused revert, the same target and
    suspended behaviour retained. A contrary event result falsifies the corresponding
    branch. Failure to reach cloak start leaves the branch untested; it is not evidence
    for retention. Three cloak transitions per coverage condition. This fixture does
    not test underground entry or the own-house exception.

  - **(c) Infantry readiness inside firing and a noninterruptible sequence.** Name test-map art
    giving an ordinary non-deployer, non-jumpjet infantry unit a long *fire-standing* sequence and a
    standing fire-frame field (S-0107 Import mapping) of a value *F* such that the stage reaches *F*
    at least 3 updates after the sequence starts and the sequence completes at least 6 updates after
    that. Its weapon is a named test weapon with a rearm delay of *R* ticks. Keep the unit
    stationary in *guard*, human-owned, with a single enemy inside its weapon's range and no other
    enemy in its search range. The enemy is a durable vehicle of a named test type with a speed of
    0, armed with a named test weapon whose range is shorter than its distance to the unit, so that
    it does not fire at the unit and the unit's own search can make it its target (S-0309 table C
    rejects an unarmed building for a human-owned searcher). The unit fires by the firing rules
    (claim 24). A run in which the unit's search takes no target is void. *Fire-standing* is
    interruptible by S-0107's engine table T1, so its sequence alone does not block readiness. Each
    order is scripted input; record which update it precedes. In c1 to c3 each judged order is a run
    of its own, from a fresh start of the scenario with the unit in *guard* holding its target,
    because a second *attack* order on the same target queues nothing once the first has made
    *attack* current (claim 7). Record, read-only, after the last update of each tick: the unit's
    current and queued behaviour, its target, its sequence kind and stage, and its motion. Record
    per update whether the unit's current behaviour was processed in it, and per tick the shots it
    fires. From these records only, the run finds for each firing cycle the update *k* in which a
    *fire-standing* starts, the update *u* of its shot, the update *v* at whose end *stand* is
    current after it completes, and the update of the next *fire-standing* start. An order is judged
    only if, from the order to the judged update, the target is unchanged, the unit does not move
    and nothing but the order's behaviour is queued. Any other order is void, not a pass.
    - (c1) Long rearm: *R* is longer than the whole *fire-standing* sequence by at least 10
      ticks. The order is *attack* on the enemy that the unit already holds, so it is queued
      (S-0213 claim 5) and the target does not change. Windows by the update that the order
      precedes: W1, after *k* and not after *u*; W2, after *u* and not after *v*; W3, after *v*
      and not after the next *fire-standing* start. Expected (claim 11, *In the middle of
      firing*): in W1, *attack* is still queued after update *u* and current after update
      *u* + 1, which processes it. In W2 and W3, *attack* is current after the unit's first
      update after the order, which processes it. Falsified in W1 by *attack* current after an
      update before *u* + 1 (the state begins only at the shot, or the late start of *u* sees
      it ended), or still queued after *u* + 1 (the state lasts to the end of the kind or of
      the countdown). Falsified in W2 by *attack* still queued after the first update after the
      order (the state lasts to the end of the kind), and in W3 likewise (the state lasts to
      the end of the countdown).
    - (c2) Short rearm: *R* is longer than the stage delay of *fire-standing* (S-0107 claim 5), so
      the countdown ends after the stage has left *F*, and the time from the shot to the update *v*
      is longer than *R* by at least 6 ticks, so the weapon is ready again while *fire-standing*
      still plays. The order is as in c1, preceding an update after the tick of the shot plus *R* +
      2 and not after *v*. Expected: no new *fire-standing* starts before *v* (the stage keeps
      rising), *attack* is still queued after update *v*, and it is current after update *v* + 1,
      which does not process it (the late start). Its first *attack* processing comes in update *v*
      + 2, and a new *fire-standing* starts in update *v* + 1. Falsified by *attack* current after
      an update before *v* + 1 (no second begin), by an *attack* processing in update *v* + 1 (a
      first start), or by *attack* still queued after update *v* + 1. Orders in this set-up that
      precede an update after *u* and before the tick of the shot plus *R* − 2 are judged as c1's
      W2.
    - (c3) Change of target at the order: c1's set-up, with the order a *move* to a nearby clear
      cell, preceding update *k* + 1. The set-up puts the shot at least 3 updates after *k*, so the
      order comes before the update of the shot. Here the target change and the walk that follows
      are the point, so the unchanged-target and no-motion conditions do not apply: the order is
      judged if nothing but *move* is queued from the order until *move* is current. Expected: at
      the read after the order's tick the current sequence is not *fire-standing*. Whether it is
      *stand* or already a walking sequence is not judged, since the *move* processing in that tick
      may already have asked for a walking sequence (S-0107 claim 10). *Move* is current after the
      unit's first update after the order, which processes it, and the unit fires no shot from that
      *fire-standing*. Falsified by *fire-standing* still current at that read, by *move* still
      queued after that update, or by a shot after the order and before *move* is current.
    - (c4) Sequence noninterruptibility: an infantry deployer whose named art gives *deploy* a
      long frame count. Order deployment, then a move while that sequence plays and no firing
      or motion is in progress. *Deploy* is noninterruptible by the same engine table; the
      test art changes only its duration, not that attribute. Expected: *move* stays queued
      until sequence readiness permits its start, without waiting for the old processing
      timer. Starting during the sequence, or not starting at the first otherwise-ready start,
      falsifies the rule. Readiness here concerns when *move* starts, not its later
      deployed-infantry destination outcome.

    A refused input does not test queue readiness. Collect three judged runs in each of W1,
    W2 and W3 of c1, three for c2, three for c3 and three orders for c4. A window with fewer stays
    inconclusive; it is not inferred from another window.

  - **(d) SAM secondary range bypass.** Extend B16 l with a named secondary weapon
    whose maximum range is shorter than the primary's, retaining its ability to hit
    aircraft and a distinguishable projectile. Keep the airborne target beyond that
    secondary range but inside the primary range; verify both ranges by S-0307 and
    record the primary firing-check result, both weapon attempts and both projectiles
    on each due aimed processing. Expected: a primary *can fire* still produces both
    shots, notwithstanding the secondary's range and rearm begun by the primary.
    Missing the secondary shot on such a processing falsifies this bypass. Retain
    B16 l's in-range secondary control. Record 20 firing ticks per condition. This
    does not establish unchecked firing for every other refusal category.

  - **(e) Non-SAM ammunition exhaustion.** Extend B16 a with named test-map rules
    giving the building a maximum ammunition of 0 and no initial-ammunition value, so
    that every attempt answers *out of ammunition* (S-0301 claim 10). Keep its durable
    target in range and its facing settled. Record
    ammunition, rearm, firing-check result, ready mark, slots and processing expiry
    around each processing from the first attack order, through the first *out of
    ammunition* answer. Expected at that answer: target cleared, *guard* current by the immediate
    start, and 1-tick processing delay without a delay draw. Retaining *attack* or
    target, merely queueing *guard*, or another delay/draw falsifies the rule. A run
    in which the firing check never returns that answer cannot test this branch.
    Three valid exhaustion events; compare an otherwise identical unlimited-ammunition
    control, which must not return to guard on that ground.

  - **(f) Force without ownership change.** Reuse S-0405 claim 13 and W8 a's
    control-centre switch fixture, retaining its stock vehicle/building types and power
    conditions. First move the dependent vehicle away from all building cells and let
    it retaliate under B7 b, leaving *guard* suspended and *attack* current. Switch off
    its owner's last active control centre while the attacker stays alive. Record
    operation count, disabling/enabling, owner, slots, target, destination, guard spot,
    start tick and processing expiry around the power transition. Expected at disabling:
    owner unchanged, *guard* current, queue empty, target/destination/guard spot clear,
    processing timer expired now and start tick reset, but the suspended *guard* still
    present. A cleared suspended slot falsifies retention across force; missing guard
    reset effects falsify that reset site. With the attacker still alive, switch the
    centre back on: expect a second guard reset with the same retained suspended slot.
    Three switch-off/on pairs. No takeover or target-loss event may substitute for the
    switch; a run without the control-count transition tests neither reset site.

- **B46 (claim 11: late start in the expiry update of a berserk infantry unit; S-0610 P9 part b).**
  Proposed measurement, not a result. Use the setup of S-0610 P9 part b for infantry: a
  human-owned infantry unit made berserk by a berserk-causing hit, an enemy durable unit far
  outside its weapon range but in its reach, the unit ordered to hunt that enemy unit before the
  hit (a berserk unit takes no orders, S-0610 claim 6, and the hit only queues *hunt*, which takes
  effect at the unit's next behaviour change, S-0610 claim 4), the common skirmish setup, speed
  index 2, no external controller, read-only recorder. The recorder reads the unit's current and
  queued behaviour, its moving state and the expiry tick (the first tick on which the berserk tint
  of S-0610 claim 7 is gone and an order is accepted) after the last update of every tick and again
  before the first update of the next tick. A run is valid only if the unit is moving and ready
  (the infantry readiness of this spec) on the expiry tick, and is not being erased, not in a
  tunnel, not in its leaving stage and not in the middle of firing; any other run is void, not a
  pass. Repeat until three valid runs are recorded.
  Expected, taking claim 11's example ("a queued *guard* can become current at the late start") as
  the outcome of a valid run: on the expiry tick the unit keeps moving and takes no guard-mode action
  (search, hold); at the read after the last update of the expiry tick *guard* is current and
  nothing is queued; the reading before the next tick's first update agrees. Falsified by *hunt*
  still current (with *guard* queued) at the read after the last update of the expiry tick while
  *guard* is current at the first update of the next tick, which shows that only the first start
  of the next update makes it current. Also falsified by a guard-mode action on the expiry tick.
  If *guard* is neither current nor queued at either read, the run shows a different expiry rule
  and is reported to S-0610. This part does not test any other late start.

- **B47 (claim 11: a vehicle whose body is still turning is not ready).** Proposed measurement,
  not a result. Setup: a tiny flat map, one human-owned turret-less driving vehicle,
  a durable enemy structure without a weapon, stock data, speed index 2, with no external
  controller or mods. Use scripted input and a read-only recorder, five valid runs. Begin with *guard*
  current, no movement and no body turn. Put a free adjacent cell opposite the placement
  facing, with the structure farther along that same direction but within weapon range.
  Order the move, then issue attack on the first tick of its pre-step turn.

  Record the order boundary and each tick's end: current and queued behaviour, body facing
  and target facing, turn start and duration, processing timer, behaviour start tick,
  order-level and locomotor destinations, heading point, cell and movement step. Keep
  runs whose state immediately after order application, before the vehicle's next first
  start, has *move* current, *attack* queued, no heading point or locomotor destination,
  no displacement and an unfinished body turn. These are input-state qualifications,
  not permission to discard an unexpectedly early start. Record other initial states
  separately against S-0207 claim 11 and its scenario D11 c, which answer Q-1160
  question 2's cancellation consequences; do not assume that cancelling the goal erased
  every movement state. An independently caused approach destination, body re-aim or other
  readiness blocker makes the later isolated comparison inconclusive. A change caused by
  an unexpectedly early *attack* start is instead a falsification, not a void run. A target
  along the turn's existing target direction avoids a facing-error change to a different
  body target. If no ordinary input reaches the qualified state, record B47 untested.

  Let *t* = *t*₀ + Θ for the recorded unchanged turn (S-0206 claim 1). Expected:
  *attack* stays queued for tick numbers below *t*, but becomes current and processes
  in tick *t*. Its start tick is *t* and its processing timer at tick end contains the
  chosen attack delay rather than the expired-now timer of an unprocessed late start
  (claims 3 and 6). For this timer comparison require a positive chosen standard delay,
  recording its value rather than assuming it. Record the scenario random stream as a corroborating delay-draw
  witness. A shot alone is not proof that *attack* processed: ordinary vehicle firing
  can run while another behaviour is current. Falsifiers: a start before *t* with the
  body turn unfinished, no start or no attack processing in tick *t* with no other
  blocker, or a processing delay inconsistent with claims 6 and 23. In particular,
  a timer left expired-now in tick *t* followed by the first attack processing in
  tick *t* + 1 falsifies same-tick processing. A facing that changes from
  its previous tick's value to its target in tick *t* is already finished at that tick's
  first start and does not falsify the rule.

- **B48 (claim 11, Q-1161: a movement goal is not sufficient to block a start).**
  Proposed measurement, not a result. Use a tiny flat stock-data map at speed index 2,
  with no external controller or mods, one human-owned driving vehicle, a durable enemy target and
  friendly vehicles that can close a one-cell passage. Script input and record order
  boundaries and each tick's end with a read-only recorder. Three valid runs per case.
  Record both behaviour slots, processing timer, body turn state, order destination,
  locomotor destination, heading point, integer movement step and position.
  - Fresh move: begin stationary in *area guard*, then repeat from *attack* with a target
    after aiming has finished, and from *sleep* after an ordinary repair-depot arrival
    under S-0314 claim 17. Include a stock repair depot for that last case and wait for
    the body's turn to end; repair-depot contact is not vehicle-factory contact. Order a free cell five cells away. Require the old heading
    point to be absent and no body turn at the first start. Expected: the order boundary
    already has a locomotor destination, but *move* becomes current and processes at the
    next first start, before arrival. A due old area-guard processing does not run first
    and send it back to its guard spot. Falsified by *move* not starting and processing
    at that first start despite the verified readiness conditions, in particular by
    keeping it queued solely until that destination disappears. If the depot run has another readiness blocker, repeat
    after it ends, or report that comparison inconclusive rather than forcing *sleep*.
  - Retained-goal wait (conditional investigation, `unmeasured` in claim 11): no
    legal order preserving all prerequisites is established here. Try closing the passage after a move order and record a retained-goal
    blockage or retry wait of S-0207 claim 25. Use an accepted ordinary behaviour order
    that leaves a nonempty movement goal; record the actual references rather than
    assuming an attack order retains the old goal. If a legal sequence reaches the state,
    preserve that sequence and repeat it three times. With no body turn, and either no
    heading point or integer movement step zero, expected: the queued behaviour starts
    at the next eligible start despite the goal. If the recorded heading point remains
    present and the step is positive while S-0206's locomotor-moving test holds,
    expected instead: it stays queued unless claim 11's ordinary exceptions apply.
    Falsifiers in a validated fixture: a behaviour remaining queued at an otherwise
    eligible start with no body turn and either no heading point or zero step, or a
    behaviour starting while all three positive-step movement conditions hold and no
    readiness exception applies. A turn, contact, transition or readiness exception makes a run
    unsuitable for that comparison. If no legal order retains the needed state,
    report that subcase inconclusive; do not synthesize a queued slot.

- **B49 (claim 22, Q-2656: the infantry end-of-update check).** Proposed measurement, not a result.
  M-0023's map and settings, stock data. An infantry unit *I* of a human-controlled house, of a type
  that does not default to area guard and without the guard-area veterancy ability (so claim 14
  queues *guard* for it), rests in *guard* on open ground, a few cells from the walled pen and on no
  factory's cells. (a) Give *I* an ordinary move order to the pen's centre, a cell in another reach
  region, taking effect at the start of a tick. The harness gives every move order of this scenario
  directly, as in M-0023, not through a player's input: an order given at the interface would have
  the pen's cell replaced by a cell in reach (S-0214 claim 11). Record at the end of each tick, for
  20 ticks from the order, *I*'s current and queued behaviour, destination and position. Expected:
  at the end of the tick at whose start the order takes effect, *move* current, *guard* queued and
  no destination; at the end of the next tick *guard* current, nothing queued and no destination; no
  step in the 20 ticks. Falsified by *move* current with nothing queued at the end of the order's
  tick, or by *guard* becoming current later than the next tick. (b) As (a), with the order to a
  free cell 5 cells away on open ground. Find the first tick at whose end *I* has no destination:
  its walk ends on its arrival in the destination's cell, and the destination is cleared there
  (S-0208 claims 6 and 16). Expected: *move* current and *guard* queued at the end of that tick, and
  *guard* current at the end of the next. Falsified by *move* current with nothing queued at the end
  of that tick, or by *guard* becoming current later. (c) As (b), with a stop order while *I* walks.
  Its arrival update (S-0208 claim 6) is the tick of its last change of position, when the arrival
  places it on its walking target, or the tick after it, when the last step ends exactly there.
  Expected: *move* current and *guard* queued at the end of exactly one tick after the stop order,
  which is the tick of *I*'s last change of position or the tick after it, and *guard* current at
  the end of the next tick. Falsified by no such tick, by such a tick later than the tick after the
  last change of position, or by *guard* not current at the end of the tick that follows such a
  tick. (d) A separate game on open flat ground: an infantry unit of a type that does not default to
  area guard, without the guard-area veterancy ability, of a human-controlled house, where a named
  test rules file sets that type's locomotor field (S-0207 claim 14) to the teleporting locomotor's
  class identifier (S-0207 table T3) and changes nothing else, rests in *guard* and is given a move
  order to a free cell 5 cells away. Find its jump tick, the tick in which its position changes to
  the landing cell. Expected: at the end of the jump tick *move* current, *guard* queued and no
  destination. Falsified by *move* current with nothing queued at the end of the jump tick.

## Open questions

- **Other cases of the end-of-update check (claim 22).** When the check finds an infantry unit in
  *move* not moving but holding a destination, the original sets that destination again (medium for
  the setting itself, `unmeasured`: no scenario here can hold an infantry unit in *move*, not
  moving, with a destination, at the end of an update). What that does to its path and to a running
  retry delay (S-0212 claims 20 and 21) is not traced, and our engine does nothing in that case,
  which matches the original only if setting the destination again has no effect there (low).
  Whether a vehicle in *move* has a like check is not traced either. Our engine makes the check for
  infantry only.
- **A team member under a team script's move step (claim 22).** Which behaviour a team script's move
  step makes current for an infantry team member was not traced. If it is *move*, the end-of-update
  check catches the member at the end of the update in which its search fails at the gate, and claim
  14 row 12 queues *guard* for it there, so S-0212 scenario P7 run 7b's "no behaviour is queued"
  would hold only for the gate failure itself, not at the end of that update (low). Our engine has
  no team members, so it makes no choice here.
- **A berserk hit and the no-threat fall-back.** A hit that makes its target go berserk reports
  *unchanged* without lowering health (S-0302 claim 10). Whether it makes an actor in a no-threat
  behaviour fall back to rest (claim 16), as a hit that lowers health does, is open here: S-0302
  claim 12 states that a berserk hit has none of the reactions.
- **Claim 39 and a later revert.** When an ordinary hit makes the vehicle retaliate under an
  *attack* override (claims 10 and 18) and the crush response then gives it the attacker as its
  destination, whether a later revert of that override brings back the destination put aside
  before the hit over the response's destination is not established. Our engine applies claim 10:
  the revert brings back the destination put aside. Whether the original does is `unmeasured`,
  medium.
- **Q-0238 hand-off:** claim 38 and F11–F12 now complete the centre, radius and no-base
  geometric answer, also answering Q-0312. S-0212 claim 23a supplies the same-tick queue
  and destination rules and final nearby search. Keep Q-0238 open until the
  complementary revisions are approved together; no request status is changed here.

- Whether any stock map, campaign mission or team script puts an actor into *queued move* or
  *attack move*. The program itself never makes *queued move* current, and makes *attack move*
  current only through the attack-move orders that S-0213 claim 6 translates to that standing
  behaviour (claim 12). Ordinary eligible input creates target-only or destination-only orders,
  neither of which translates to that standing behaviour; no ordinary input path for it is
  established. Confirm this distinction with S-0213 claim 6 at joint sign-off.
- Whether an attack-move order can reach a building in the original. Buildings are never eligible
  for attack-move (S-0214), so input does not create one; if one arrived, the building would queue
  *attack move* (claim 12).
- Whether any stock situation reaches the out-of-stage rule of a SAM building's *attack* (claim 32):
  a building with the SAM flag in a behaviour whose rule leaves the progress count outside 0 and 1
  (a gate's *open* in its stages 2 to 5, or a *repair* or *missile* processing), followed by an
  override to *attack* (for example a retaliation) while that value stands. Scenario B16 (m) reaches
  the rule only with a test gate type, and only if the retaliation rules allow the override while
  the gate is in *open*.
- Which additional structures rules set or clear a building's ready mark (claim 11), beyond
  the established processings, the body-range rule and the grand opening (claim 1). An unarmed building's own *guard*
  processing never sets it, but its non-turreted body range can. Factory release and unloading
  neither set nor clear it: the queued *unload* and the *guard* queued at its end wait for
  claim 11's ready-mark rule, with the body-range consequences of S-0417 claim 2 and
  `quirks.factory_unload_waits_ready_mark` (default original). The factory case is resolved.
- The guard spot's reference point when the guard spot is a cell (F4): presumably the cell's centre
  at the cell's height.
- Which situations fill a vehicle's approach destinations (claim 13). S-0417 claim 5 now
  establishes factory-exit temporary driving for teleporting and tunnelling vehicles.
  Other temporary replacements remain outside the established setups here.
- Whether an actor that changes owner (for example by mind control) in the middle of a route keeps
  following that route (claim 13, whose route). If it does, the original takes the route's next
  point from the routes of the house played on the machine, which in a skirmish differ from the
  new owner's.
- Which actors are marked as loaners (claim 15, row 5), and whether any fall-back other than the
  placement fall-back skips the land-type test of claim 15, row 4.
- The absent-key default of the auto-deploy delay list: no engine default is known. (The guard
  stray distance has none in the original; claim 36 b gives our engine's replacement.)
- Aircraft readiness and queued-start opportunities are now owned by S-0501 claim 31,
  with ordinary-flight coverage in AC10. The unisolated exclusions remain as that claim states,
  not an unspecified list of flight phases here.
- The bunker rule of claim 17 and its exact conditions are owned by S-0606 claim 6.
  It is not the only identified threat-point sender: S-0417 claim 4's factory apron-neighbour
  clearing supplies another setup for the paralysed refusal (scenario B6, part c).
- The vehicle *unload* processing for deploying vehicles, *retreat* and *patrol*: claims 30
  and 34 point to other topics. S-0507 claims 7–12 own transport unloading.
- **Q-0641, depot coordination (claim 29).** S-0314 claims 16 to 24 now own permission,
  contact, completed arrival, order interruption, queued entry, sale/destruction, existing-contact
  power consequences and the absence of automatic player-house repair return. Claim 17 there
  owns the separate multi-place servicing qualifications. Retain those contracts with claim 29
  at sign-off, rather than treating them as unresolved here. The further vehicle-state condition
  for sending a run-away scatter request remains open in S-0314 claim 22. Claim 18 there
  owns the docking-point selection and cell-based stopping, including its stated vertical
  boundary and measurement limitations; behaviour processing alone supplies no further
  approach geometry.
- The refused deploy of claim 27 (vehicles): which situations let a deploying vehicle in *hunt*
  reach a spot where its building may not then be placed, so that the refusal and its queued
  *guard* can be observed. The deploy specs own the placement test.
- The *enter* processing (claim 29) can, in some cases where the actor has nothing left to enter,
  fall back to rest and start the resting behaviour at once before choosing its standard delay,
  which would then take the rate of the started behaviour (claim 6). The entry protocol's specs
  own those cases.
- **Q-0072, question 2 (exceptional searches).** S-0307 claims 15 to 25 cover ground
  candidates on terrain and bridges, cell targets including destroyable cliffs, failure centres
  after trials, usable-area exceptions, balloon-hover destinations, close-range searches,
  the jumpjet detour exemption and the non-walking shortcut response. Only the deploying-search
  gap remains.
- Which parts of the approach fill a vehicle's approach destinations (Terms; claim 37 f).
- The in-the-playfield mark (Terms; claim 37 f): which objects other than ones arriving from
  outside the usable area lack it.
- The *sticky* and *any other* rows of claim 37 d cannot be told apart by observation from the
  per-tick drop of claim 37 a, the only way an actor in those behaviours reaches the step, so they
  stay unmeasured; the *attack*, *area guard* and human-controlled *hunt* rows are backed by
  scenario B32 (c and c2).
- **Q-1154, question 2.** Claim 24 counts a unit already waiting in line for the found
  absorbing building as "already bound", so the processing does not send it again.
  This remains low-confidence and `unmeasured`: B21 d does not establish the needed
  legal waiting-line fixture. Validate that input sequence before using the proposed
  expected result and falsifier to settle the question.
- Whether the crush response (claim 39) still queues *move* when the vehicle's new destination
  is refused, because it is deployed, deploying, undeploying or seized, is not established. With
  no destination, a current *move* falls back to rest (claim 22), so the two outcomes differ only
  in the behaviour that the vehicle passes through.

### Cross-spec changes to confirm

- S-0402 (or the economy spec that takes it): the original's return of a damaged resource-gathering
  vehicle to a refinery after a hit, for a computer-controlled owner, is not specified. When it is,
  it should say that it is skipped after a hit that gives the crush response, and so only where the
  response is made: the skip shares the response's conditions on teams, the tether, alliance and
  the owner only as a consequence of that (claim 39). The skip is
  `unmeasured`, medium, as claim 39 rates the harvester ordering.
- S-0309 and S-0610: M-0011 reports that a berserk fortress held as its target the drone that it
  then drove over. The crush response sets only the destination. Whether the opportunity search
  of *move* (claim 33) can take a unit of a berserk-friendly type decides where that target comes
  from. No change is requested: the source of that target stays open (S-0210 Open questions).

- S-0206 Terms already distinguishes its locomotor-moving query from claim 11's readiness
  query, and S-0603 claim 3 already delegates readiness here. Retain those distinctions.
  S-0206 should clarify that the readiness query is not merely its own query plus body
  turning: a non-turning driving vehicle additionally needs a present heading point and
  a positive integer movement step to count as moving for readiness. Its claims 9 and 16
  need no widening.
- Q-1160 question 2 is now covered by S-0207 claim 11 and its scenario D11 c:
  cancelling the pre-step destination leaves no heading point, cannot authorize the
  cancelled step and does not itself stop or retarget the body turn. Retain that
  matching contract, separately from B47's aiming and readiness qualifications.
  No further S-0207 change is requested.
- S-0402 claim 11's turning-body discussion and its later turn-end-late-start
  outcomes should use claim 11 here: without another blocker, a tick-number turn
  expiry permits that tick's first start and same-tick processing, not just a late
  start. This replaces the earlier cross-spec note requesting late-start timing.
  S-0314 claim 17 agrees with claim 29: ordinary depot arrival queues repair/sleep
  without requesting a map-east turn, even when tethered. Only an actual existing driving
  turn can impose claim 11's body-turn readiness hold; ship and hovering body rotation
  alone cannot. Retain the matching contract; no further change is requested there.
- S-0603 claim 3 should distinguish the movement state read at the late start from
  a remembered displacement earlier in the update. For a driving vehicle, apply the
  complete current predicate here, even after an update that moved the unit.

- S-0009 claim 40, S-0108 claim 11 and S-0402 claim 18 already agree with claim 1:
  set-up queues *guard*, the grand opening sets readiness during placement, and the
  first per-tick start makes *guard* current. Those claims own the opening's other
  effects. Retain these matching contracts at sign-off; no further edit is requested.
- S-0301 claims 9 and 14's current corrections agree with claim 32's immediate upgrade
  pair, empty-secondary no-op and priority over animation delay. S-0406 claim 12 agrees
  on the position-1/count gate; confirm these contracts at joint sign-off.
- S-0314 claim 17 already qualifies same-cell retention as a building destination,
  citing S-0207 claim 26's cell-goal cleanup and exact-point guard exception. Claim 29
  here supplies accepted-entry processing retention, not an exemption from other entry
  effects. Retain the matching contract at sign-off; no further edit is requested.
- S-0308 claim 12 already uses S-0406 claim 12's count-limited position order, not
  installation history, and distinguishes that provider scan from claim 32's
  position-1-only gate. Its W9 paired-attempt tests cover slots 0 and 1 separately from
  permission. Retain those matching contracts; no further edit is requested.
- S-0212 claim 23a's backing already inherits claim 38's unmeasured qualifications for
  separately set centre precedence, start-assigned records and price-factor-only
  changes: B37 supplies no isolated legal setups. Retain those qualifications; its
  covered geometry, draw and failure-timing backing is unchanged.
  S-0212 claim 23a and its Q-0238 hand-off question already cite
  claim 38 and F11–F12 here for the complete centre geometry and no-base usable-area
  correction. S-0212 claim 25 also supplies the separate base-record-refresh search
  with claim 38's parameters. Retain these matching contracts in its current correction draft and confirm
  them at joint sign-off; there is no remaining centre-selection gap. Q-0238 must remain
  open until the complementary revisions are approved together.
- S-0309's current claim 23 b already cites claim 13 here for assignments that can supersede
  the refresh result and for route-generated orders rather than immediate destination changes.
- S-0610 claim 5 and P9 part b already cite B46 for the expiry-update late start of
  moving, ready infantry, including its expected value and falsifier. Retain that shared
  proposal and the unmeasured qualification for other expiry cases; no further edit is
  requested there. Its start-order question remains open until B46 is run.
Joint hand-off checks: S-0213 claim 16 already supplies the route-order timing
pointer. Confirm S-0207 claim 20 e and S-0208 claim 19 with claim 10's airstrike exception,
not an erasing-lock exception, and its restored-target distinction. S-0307 claims 15 to 25 and S-0406
claim 17 and S-0417's current claims 2, 4, 5, 13 and 14 supply the expanded search,
construction, factory ready-mark and temporary-driving contracts cited here. Hand these corrections off together; no further
behavioural edit to them is requested. Also confirm S-0207 claim 20 a/e with S-0209's
matching ground-reservation and bridge-record consequences and their shared
`quirks.teleport_vehicle_ground_reservation`; S-0212 claim 22 with S-0207 claim 15 and
D17 d3 for immediate fresh search and same-tick follow-up; and S-0208 claim 19 with
claim 10 here for teleport landing and target-loss/reversion. Retain claim 2's locomotor
phase-in processing and S-0513 claim 51's distinct superweapon-warp design exclusion through
arrival effects, with ordinary eligibility after the arrival-hold expiry test; retain
claim 11's unchanged factory ready mark and claim 13's restoration hand-off
to S-0207 claim 21; S-0417's release and temporary driving do not supply exceptions to
them. Confirm claim 33's charging break-off exception with S-0309 claims 1 and 4 and
S-0518 claim 15. Also retain claim 37's approach trigger, destination retention and processing
contract with S-0307 claims 15 to 25, S-0212 claim 25's failed-approach parameters and
S-0208 claims 3, 17 and 21's position-choice and walking outcomes. These are coordination
checks, not additional rules of this spec.

## Changes for other specs

- **S-0212, claim 23 (the gate row).** Its "no behaviour is queued" holds for the gate failure
  itself. An infantry unit whose current behaviour is *move* then falls back to rest at the end of
  the same update (S-0215 claim 22), so what claims 13 and 14 queue, usually its rest behaviour, is
  queued at the end of that update and current at the first start of the next tick if it is ready
  there. The gate row should point to S-0215 claim 22 for this, and scenario P7 run 7b, whose
  Expected gives "no behaviour is queued" for a team member after the gate failure, should record
  its current behaviour and point to S-0215's Open question on team members.
- **S-0417, claim 17 (*No path*).** For an infantry unit in *move* whose search fails at the gate,
  the same holds: what claims 13 and 14 queue, usually its rest behaviour, is queued at the end of
  that turn and is current and processed at the first start of the next tick if it is ready there
  (S-0215 claim 22), not after the *move* processing's standard delay. The bullet for a search that
  fails at the gate should say so.
- **S-0207, claim 20 (e).** Its "but not within the jump tick: the behaviour is processed before the
  locomotion (part d), so the fall-back comes at a later tick, at the behaviour's next processing or
  at the end of phasing in with no target (part f), whichever is first" does not hold for an
  infantry unit. An infantry unit whose *move* is current falls back to rest at the end of its jump
  tick by S-0215 claim 22's end-of-update check, since the jump in its own locomotion stops its
  movement and removes its destination before that check: its rest behaviour is queued at the end of
  the jump tick (S-0215 claim 22, medium, `measurable` by scenario B49 part d). For the other kinds
  the sentence is unchanged. Claim 20 (e) should except infantry from the sentence, as the claim
  does elsewhere where it names infantry, and point to S-0215 claim 22 for it.
- **S-0309, claim 23 (d), its Backing, its Import mapping row and scenario A6 parts n and o.** For
  an infantry unit, the uncontested arrival is caught by S-0215 claim 22's end-of-update check at
  the end of the update whose locomotion ends the walk, after that update's late start and before
  any later *move* processing. That fall-back's refresh removes the memory (S-0309 claim 23 (b),
  S-0215 claim 13), and the rest behaviour of S-0215 claim 14 is queued at the end of that update
  and current at the first start of the next tick if the unit is ready there (S-0215 claim 22,
  medium, `measurable` by scenario B49 part b for an arrival under a plain move order, and by A6
  part n's record for the attack move). So no *move* processing of the infantry unit chooses 1 tick
  at that fall-back. These places give the 1-tick delay at the arrival and hold for a vehicle only:
  part (d)'s "Its next *move* processing falls back to rest and chooses an integer delay of 1 tick
  (S-0215 claim 22)", the claim's Backing "part n for arrival, memory removal and the 1-tick delay",
  the Import mapping row "claim 23's arrival delay, memory refresh and removed-target conversion |
  fixed behaviour and integer 1-tick processing delay", A6 part n's "and that processing chooses 1
  tick", and A6 part o's run 2 "arrival clears the memory as part n gives, with a 1-tick
  arrival-processing delay". Each should name the end-of-update check for an infantry unit, and
  parts n and o should expect an infantry attack mover to fall back at the end of its arrival
  update.
- **S-0302, claims 10 and 12.** They say that a berserk hit sets off none of the reactions that
  claim 12 ties to a result ("a berserk hit has none, whatever it reports"). The crush response
  (claim 39) is not tied to the result and is made after a berserk hit too. S-0302 claims 10 and
  12 should say so where they list what a berserk hit does not set off.
- **S-0307, Open questions ("Reach region of a moving attacker").** It names the point whose
  cell the clear-cell test reads the scatter reference point, "its own cell only while it is not
  moving". For infantry that is exact (its walking target, else its own position). For a
  vehicle the point is its heading point as its movement spec gives it, as for claim 37 (a1)
  here, which S-0208's term (built on an infantry unit's walking target) does not name. The
  question should name the point as claim 37 (a1) does.
- **S-0610, claim 4.** The *hunt* that the berserk entry queues is replaced by *move* when the
  same hit gives the crush response (claim 39). Its re-hit sentence ("A berserk hit on a unit that
  is already berserk only sets its remaining time to the new *T* … Nothing else changes: its
  target and behaviour stay") holds for the berserk effect only: the same hit's crush response
  then gives the crusher a destination and queues *move* (claim 39). S-0610 should point here for
  both cases.

- **S-0610 (approved, revision 11), claim 5, "Which updates are countdown updates".** In the
  *Excluded updates* bullet, after "or is in the leaving stage of a chrono move (S-0513 claim
  39)", add: ", and the updates in which a superweapon-warp member phases in that claim 2 here
  excludes: those of a vehicle or landed aircraft member with a temporal beam attached, being
  erased or not, and those of an infantry member while it is being erased or, with S-0513
  `quirks.phase_in_beam_spares_infantry` off, while a beam is attached to it (S-0513 claim 39a)".
  In the *Warp-only updates* bullet, after "whether the hold is still holding or expiring, as
  S-0513 claim 51 gives the other ordinary work", add: ", except in those excluded phasing-in
  updates, which take no countdown step and keep the remaining time". Level: medium, and low for
  a landed aircraft member, as S-0610 claim 5 rates the countdown of a landed aircraft being
  erased (its Open questions) and claim 2 here rates the landed-aircraft tick. Backing:
  `unmeasured`, since no scenario makes a berserk warp member phase in with a beam attached; the off
  setting of the switch is design.

- **S-0208 (approved, revision 34), S-0309 (revision 116), S-0107 (revision 40) and S-0607
  (revision 39): the no-threat fall-back within a hit (claim 16 here).**
  - S-0208 claim 15 should point to claim 16 here for the place of the fall-back between its
    requests A and B: request A comes before it and request B after it.
  - S-0309 claim 16 (anger) and S-0107 claim 16 (fear) should point to claim 16 here for their
    order against the fall-back. Neither rule changes.
  - S-0607's *A fall-back to rest in the grip* should name the hit that triggers it as claim 16
    here gives it, in place of "survives damage from a known attacker". A gripped unit never
    retaliates (S-0607 claim 9), so the order against retaliation changes nothing there.
  - Level and backing: those of claim 16 here.
- **S-0107 (approved, revision 31), claim 11, the switch `quirks.prone_attack_state_persists`
  and scenario M8 parts (a) and (d).**
  - *End of the attack state* should list the shot first: when the weapon fires at the fire
    frame, the attack state ends with that shot (claim 11 here, *In the middle of firing*). It
    begins again only on a later tick that meets claim 11's entry, so at or after the end of
    the rearm countdown.
  - The sentences "When a firing kind completes and its follow-up (State and ordering) is *stand* …
    once its weapon is ready" and "A prone unit's follow-up, *prone*, has a non-zero delay, so it
    stays in the attack state" hold only for a unit that is in the attack state when its firing kind
    completes. That is a unit whose firing check answered *can fire* again while the kind still
    played, so that the attack state began again with the request turned down, or one whose kind
    completed before its stage reached *F*. They should say so. A unit whose rearm countdown
    outlasts its firing kind is not in the attack state when the kind completes, and its next shot
    comes from a new firing kind. The repeat shot during *prone* and the switch keep their content
    within that scope. The switch's description should begin "A prone unit that is in the attack
    state when *fire-prone*, or *secondary-fire-prone* that is not repeated for an airstrike (State
    and ordering), completes stays in it", since the follow-up of such a kind is *prone*, and its
    off setting should end a prone unit's attack state when such a kind completes, leaving a
    standing unit's ending as it is.
  - *When its target changes* should add that the change, to another target or to none, also asks
    (not forced) for *deployed* when deploying, for *prone* when prone and for *stand* otherwise, as
    the not-ready fire frame does, whether or not the unit is in the attack state at that change, so
    a firing kind still playing gives way to it.
  - M8 (a) should judge the prone repeat shot only in runs whose rearm countdown ends before
    *fire-prone* completes, and add a run whose countdown outlasts it. Expected there: every
    shot comes from a new *fire-prone*, falsified by a shot during *prone*. M8 (d) expects the
    running sequence to go on with its stage not restarted after the order to attack another
    target. With the request above, the order replaces a *fire-standing* still playing by
    *stand*, so M8 (d)'s Expected should say so, and its facing judgement should be checked
    against claim 11's entry rule.
  - The repeat shot's "at most once per start of *prone*" then follows from the end at the shot: the
    shot ends the attack state, and a later begin asks for a new firing kind. The repeat-shot
    paragraph and the switch's description should say so in place of their reasoning from the later
    ticks of the stage value. M8 (a)'s run whose countdown outlasts *fire-prone* should end the
    countdown after *fire-prone* completes and before the stage of *prone* could reach the prone
    fire frame, since only then would a unit still in the attack state fire during *prone* (S-0107
    revision 40 gives that window in ticks). *When its target changes* should name the unit's
    per-tick search (claim 33) beside its own processing.
  - Level and backing: those that S-0107 revision 40 gives in claim 11's Backing and Confidence
    lines, which carry these changes.
- **S-0513 (approved, revision 115), claim 39a (*The firer*) and scenario SW12 parts (l9a) and
  (l9b).**
  - The deploy order clears the unit's target, and that change of target ends the attack state
    as the order takes effect (claim 11 here, *In the middle of firing*). A shot has already
    ended it as well, so no attack state "that a shot left" remains. The unit is therefore not
    in the middle of firing at either per-tick start of its first update after the order.
  - Replace the sentences from "A unit with no target is no longer in the middle of firing" to
    "comes before it" with: "The deploy order's change of target ends the attack state as the
    order takes effect (S-0215 claim 11), so the unit is not in the middle of firing at its
    first update after the order. Unless another readiness condition of S-0215 claim 11 holds
    then, *unload* becomes current at that update's first per-tick start and is processed in
    it, before any *attack* processing."
  - The timing that claim 39a gave, an end after both per-tick starts, is the no-target ending
    in the firing turn (claim 11 here). The deploy order does not reach it. Of the routes this
    spec traces, only S-0309 claim 23's attack-move discard does.
  - In SW12, (l9a)'s Expected can be sharpened to *unload* processed in *T*'s first update after
    the deploy order: just after that update its deploy has started and *guard* is current, since
    that processing forces *guard* (claim 30). Its exception for a run in which *T* is in the middle
    of firing just before that update counts no run and can go. (l9b) as written counts no run,
    because neither the lock shot nor the deploy order leaves *T* in the middle of firing. It should
    be dropped, or rebuilt on another readiness condition of S-0215 claim 11 that the deploy order
    leaves in place (a sequence that is not interruptible, or falling).
  - Level: medium for the end at the order, as S-0107 claim 11 rates its end conditions, until B45
    run c3 runs. Backing: scenario B45 run c3 here for an order's change of target. SW12 (l9a) as
    sharpened tests the start of *unload* that follows, not the end itself.

## Questions for the project lead

- Claim 39 and the glossary entry *AI house*: the crushers of every player house, AI houses
  included, follow the human-controlled rule and so never make the crush response by default.
  The arena behaviour of M-0011 (fortresses driving over the drones that gas them) then appears
  in our engine only between test-only computer-controlled player houses, and our AI gets it only
  by giving move orders. Decided by the glossary's *AI house* entry; recorded here for the lead.
- `quirks.auto_crush_computer_only` (S-0210, Edge cases), default on as the original: the
  auto-crush flag and the player-auto-crush rule have no effect. Decided under the autonomy grant
  (2026-10-28); recorded here for the lead.

- Claim 31, a too-short auto-deploy delay list for the neutral and special houses: the spec
  states our engine's result (importer warning, the row does not apply). Confirm it, or choose
  another deterministic result.
- `quirks.turret_snap_bound_wraps` (Edge cases and quirks, F8): keep it as a switch of this spec,
  or leave the field's domain to a revision of S-0206?

- The can-recalculate property of claim 37 e is read from a type key that the allowed-key list
  does not yet hold (it is on the deny list). Add it to the allowed-key list so the Import mapping
  can name it, or keep the property described in words only?

Decided and applied: claim 36 (b), Q-0020 (delegated sign-off, 2026-10-10: area-guard leash and
importer warning). `quirks.named_deploy_area_fire` (default on). The SAM building's unchecked
secondary shot is owned here (claim 32). Claim 32's out-of-stage rule has no switch. The neutral
and special houses keep the original's rules (2026-10-10). Every house uses the normal difficulty
row (2026-10-10). Game end in skirmish (2026-10-11): no rule of this spec depends on it. The neutral and special
houses' house skill level is 0 in a skirmish (S-0412 claim 3, scenario K10), answering the former
open question on it.
