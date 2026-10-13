---
id: S-0407
title: Tech tree (what a house may build: prerequisites and generic groups, tech level, country lists, stolen tech, build limits, captured buildings, secret labs)
topic: production
revision: 5
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-13
measurements: []
requests: []
supersedes: []
---

# S-0407: Tech tree (what a house may build: prerequisites and generic groups, tech level, country lists, stolen tech, build limits, captured buildings, secret labs)

## Summary

Which infantry, vehicle, aircraft and building types a house may order built at a given moment: the
prerequisite rules with their generic groups, the tech level, the country and stolen-tech
conditions, the superweapon setting, build limits, how capturing buildings (factories, secret labs,
prerequisite buildings) grants tech, and when the answer is worked out again. Skirmish only.
Campaign behaviour and network games are out of scope. Production itself (queues, payment, holds,
the cancel of items that stop being buildable) is S-0404's; build icons are S-0411's.

## Terms

Shared terms used here are defined in the glossary: *house*, *country*, *side*, *type*, *type kind*,
*field*, *building*, *infantry*, *vehicle*, *aircraft*, *prerequisite*, *registry*, *registration
index*, *on the map*, *in play*, *construction state*, *tick*, *local player*, *AI house*,
*computer-controlled*, *human-controlled*, *neutral house*, *special house*, *scenario random
stream*, *game start*, *absent-key default*, *importer warning*, *quirk switch*, *departure*. S-0404's
*line*, *item*, *queue*, *could make* (Terms) are used as S-0404 defines them; S-0405's *switched on*
(Terms) as S-0405 defines it; the *house update* is S-0203's. In addition:

- **Present count.** For a house and a type: how many objects of that type the house owns that are
  on the map. An object counts from the moment it is put on the map (a building at placement,
  during its construction state too) until it leaves the map or changes owner. A change of owner is
  a leaving of the map for the old owner and an entry for the new owner, and an object that enters
  the map again (for example, a vehicle leaving a transport) counts again. It is the per-type
  form of S-0414's on-map counts (S-0414 claim 4), kept by the same events. Buildings, aircraft and
  infantry of a type with the don't-score flag (Import mapping) are never counted (claim 8). A
  vehicle type with that flag is the exception: its present count for a house is the number of
  times since game start that a vehicle of the type has entered the map as that house's, and leaving
  the map never lowers it. So a don't-score vehicle that has been on the map stays counted for good,
  even after it is destroyed or has passed to another owner, whose count it then also enters, and
  each further entry (for example, from a transport) adds one more
  (`quirks.unscored_vehicle_stays_counted`,
  owned by S-0414, which holds the rule; with the switch off a vehicle of such a type is never
  counted). Whether such a vehicle matters here is claim 8 (b). S-0414's nominal-survivor quirk
  (`quirks.nominal_survivor_count`) concerns S-0414's total of infantry on the map, where some
  infantry are not added on entry but are taken off on leaving; whether it also moves the per-type
  infantry present count is not known, and no rule of this spec reads an infantry present count, so
  it does not matter here.
- **Owned count.** For a house and a type: how many objects of that type the house owns, on the map
  or not (inside a transport, inside a building, or still an item in production: an item's object
  exists from the start of its production, S-0404 claim 3). Objects of a type with the don't-score or
  the insignificant flag (Import mapping) are never counted.
- **Ever-built count.** For a house and a type: how many objects of the type the house has received
  from its factories, never lowered by losses or captures (claim 10 says which releases count).
  Don't-score types are never counted.
- **Build limit.** A type's integer field *L* (Import mapping), a signed 32-bit integer: positive, a
  limit on the owned count; negative, a limit of |*L*| on the ever-built count; zero, never
  buildable. The most negative value, *L* = −2³¹, also makes the type never buildable, whatever its
  ever-built count, as for *L* = 0 (F2).
- **Tech-tree answer.** For a house and a type, one of *allowed*, *at limit* or *refused*, given by
  claims 2 to 10.
- **Offered.** A type is *offered* to a house while it is in the house's taken-up set (claim 1 (c)).
  Claim 1 (a) and (b), that its tech-tree answer is allowed or at limit and that a factory could
  make it, are judged only at a recheck (claim 12), when the set is worked out again. Between two
  rechecks the offer does not change, even when (a) or (b) stops or starts holding. Whether an
  offered type is at limit is judged from the current counts at every moment (claims 9 and 13): an
  offered type at limit cannot be ordered to start or queue an item. A build order requesting
  resume of its held item is exempt from that issue-time limit test (claim 13), and its own icon
  shows its item state rather than darkening (S-0411 claim 3).
- **Being sold, queued to be sold.** A building is *being sold* while its current behaviour (S-0215,
  Terms) is *selling*, that is, during its sale (glossary, *sale*). It is *queued to be sold* while
  its queued behaviour (S-0215, Terms) is *selling*, for example after a sell order (S-0213, claim
  12) whose sale has not yet started.
- **Lifted.** A type is *lifted* for a house when claim 3 or claim 4 applies to it.
- **Qualifying factory.** For a type and a house: a factory building of the type's kind that has
  been put on the map, is switched on, is alive with health above 0, is not being sold and is not
  queued to be sold. It need not be the building of claim 1 (b): the owner countries and the naval
  kind of a qualifying factory are not compared, so a switched-on factory of the kind lets claim 1
  (b) be met by another factory building that is switched off.
- **Generic group.** One of six game-wide lists of building types, named by a word in a prerequisite
  list instead of a type name: the power, factory, barracks, radar, tech and refinery groups (Import
  mapping gives the words).
- **Tech-lab list.** A game-wide list of building types (Import mapping): its buildings can be spied
  for stolen tech (claim 6) and keep their superweapon with superweapons off (claim 7).
- **Stolen-tech flags.** Three flags per house, the first, second and third, which the stock data
  uses for the three sides (claim 6 says which infiltration sets which). All start cleared at game
  start.
- **Secret lab, secret type.** A building whose type has the secret-lab flag, and the type it lets
  its owner build (claims 4 and 14).
- **Override list.** A type's list of building types any one of which, present for the house, lifts
  the tech level, country, stolen-tech, superweapon and prerequisite conditions (claim 3).
- **House tech level.** S-0009's *house's tech level* (Terms): an integer per house, taken from the
  game's tech level when the house is created (neutral and special houses have 1). S-0412 does not
  set it; it only points the game's tech level to S-0009. This spec owns only how the integer limits
  what may be built (claim 5).
- **Taken-up set.** A per-house set of types: the types whose build icon the house's strip holds
  (S-0411 claim 2). The original keeps it only for the local player, as the entries of its sidebar, and
  our engine keeps one for every player house (claim 13). It changes only at a recheck of the
  offer (claim 12, the closed list of events that call for one), and what it holds after a recheck is: a type is in the set exactly when claim 1
  (a) and (b) both hold and, in addition, either the type was already in the set before the recheck
  or the house has a *qualifying factory* (Terms) at the recheck. This admission rule assumes space in the type's sidebar tab: S-0004 claim 11 suppresses an otherwise eligible new icon when the tab is full. Whether that suppressed type belongs to the taken-up set for order legality, and when a freed slot admits it, remain open (Open question 5). No cancellation rule is asserted here for a suppressed type. A type for which (a) or (b)
  fails leaves the set (or stays out of it) even when a qualifying factory exists. A type that has left the set is *not* remembered as taken up: when it is allowed again it
  needs a qualifying factory again at the recheck that follows. A type is *taken up* while it is in
  the set. Between two rechecks the set does not change.

## Behaviour

### What is offered and what can be ordered

1. **Offered types.** A type is offered to a house while it is in the house's taken-up set (Terms).
   The set changes only at a recheck (claim 12), and what it holds after a recheck depends on these
   conditions, judged at the recheck:
   - (a) its tech-tree answer (claims 2 to 10) is allowed or at limit, and
   - (b) the house has a factory building that could make it, with the tech-tree condition, power
     and docks left out (S-0404, Terms: a factory of its kind, on the map, not being sold or queued to be sold, of the
     matching naval kind, and whose *type's* owner countries share at least one country with the
     type's owner countries, claim 11), and
   - (c) the taken-up set itself: with a *qualifying factory* as defined in Terms, a recheck leaves
     the set as follows:

     | (a) and (b) both hold at the recheck | In the set before the recheck | A qualifying factory exists at the recheck | In the set after the recheck |
     |---|---|---|---|
     | yes | yes | yes or no (makes no difference) | yes |
     | yes | no | yes | yes |
     | yes | no | no | no |
     | no | yes or no | yes or no | no |

     The admission rows assume space in the sidebar tab (S-0004 claim 11). A full tab suppresses
     a new icon; its offer membership and later admission are not established here (Open question 5).

     (Example of the last row: a naval vehicle type whose other conditions are met, while the
     house has a switched-on factory building of the vehicle kind that is not naval and no naval
     factory building: a qualifying factory exists, but (b) fails, so the type never enters the
     set.)
     So (c) matters only for a type that becomes allowed while the house has no qualifying factory
     of its kind (Terms: for example, every factory building of the kind is switched off, or queued
     to be sold): it is not offered until a recheck finds a qualifying factory.
   The offer is worked out again only after the events of claim 12, which is a closed list. Between
   two rechecks it is frozen: a type stays offered after (a) or (b) has stopped holding, and a type
   for which they have started to hold is not offered, until the next recheck. Two cases in which
   this can be seen: at the start of a sale of the house's only factory building of a kind, (b)
   fails for the types of that kind, yet they stay offered until the building's removal (claim 12,
   event (b)); the build icon also stays until that recheck, rather than disappearing one tick
   after the sale starts (S-0411 claim 2). Also, when a
   refinery-alternative vehicle without the don't-score flag enters a
   transport, a type that needed it for the refinery group (claim 8 (b)) stays offered until the
   next recheck. Whether an offered type is at limit, by contrast, follows the current counts at
   every moment: a type at limit is offered but an order to start or queue it is refused (claim 13).
   A held-item resume is exempt from this issue-time limit test; its own icon shows its item state
   rather than darkening (S-0411 claims 3 and 4). The neutral and special houses build nothing.
   - Backing: `measurable` (scenarios T1 and T2; the frozen offer by T14 runs 7 and 8,
     queued-sale factory exclusion by S-0411 S2's sale extensions)
   - Confidence: high (a and b, except queued-sale factory exclusion, medium); medium (c and the frozen offer: the taken-up set is inferred from
     when the original's sidebar gains and loses an icon, which it does for the local player only,
     and the original removes an icon only at a recheck; this spec states which types are in the
     set after a recheck, and S-0411 (claim 2) gives the removal of an icon that left the set, and
     scenarios T1 and T2 test it, including the case in which (b) fails while a qualifying factory
     exists; that a failing (a) or (b) keeps the type out of the set at the same recheck, whatever
     the factories, is not yet observed and is tested by T2 run 3; whether a factory building in its
     construction state is a qualifying factory, as the Terms definition implies, is not measured,
     Open question 2)

2. **How the conditions combine.** A type's tech-tree answer depends on whether it is *lifted*
   (Terms) for the house:

   | Lifted (claim 3 or 4) | Some condition of claims 5 to 8 refuses it | Tech-tree answer |
   |---|---|---|
   | yes | yes or no (makes no difference) | the build-limit answer (claims 9 and 10) |
   | no | yes | refused |
   | no | no | the build-limit answer (claims 9 and 10) |

   So refused takes precedence over at limit: a type that is not lifted and that one of claims 5 to 8
   refuses is not offered, even when its build limit would also make it at limit.
   - Backing: `measurable` (scenarios T3 to T8)
   - Confidence: high

3. **Override buildings.** When the house's present count of any type in a type's override list is
   above 0, the type skips the tech level, country, stolen-tech, superweapon and prerequisite
   conditions (claims 5 to 8). Its build limit and the factory conditions of claim 1 still apply.
   Only building type names count in an override list. A generic group word there is bad data:
   whether the original then counts the entry as met is unspecified (glossary, *unspecified*). In
   our engine such an entry is never met and the importer warns (a *departure*).
   - Backing: `measurable` (scenarios T3 and T13); the rule for a group word in an override list is
     `unmeasured`: the original's result is unspecified, so there is nothing to observe, and the
     rule is one of our engine.
   - Confidence: high

4. **Secret types.** When one of the house's secret labs that is on the map has the type as its
   secret type (claim 14), the type skips claims 5 to 8 like claim 3. The lab's secret type is
   lifted for the house from the moment the house owns the lab on the map (for a captured lab, from
   the capture, claim 15) until it loses the lab.
   - Backing: `measurable` (scenario T9, runs 1 and 3)
   - Confidence: high (the bypass); medium (the moments at which a lab starts and stops counting
     for its owner)

5. **Tech level and country lists.**
   - (a) A type is refused when its tech level is −1 or above the house tech level. The tech level's absent-key default is 255, so a type with no tech
     level is refused at every house tech level below 255.
   - (b) A type with a required-countries list is refused unless the house's country is in it; a type
     with a forbidden-countries list is refused when the house's country is in it. Whether a country
     is in a list is decided by its registration index in the country registry, as F1 gives (see
     `quirks.country_index_wraps` for indices of 32 or more).
   - (c) A list key that is absent or empty means no restriction. A list given but naming only unknown
     countries is an empty set: a required list of that kind refuses every house, a forbidden list
     refuses none.
   - Backing: `measurable` (scenarios T4 and T5, runs 1 to 3)
   - Confidence: high (a, b); medium (c)

6. **Stolen tech.** A type that needs the stolen tech of a side is refused unless the house has that
   side's stolen-tech flag. A type can need any of the three. A house gains a flag when one of its
   spies infiltrates a building on the tech-lab list that is not a radar building (glossary) and has no power output
   (power field 0 or below): the flag is chosen by the infiltrated building *type's* base-planning-side
   field: 0 gives the first flag, 1 the second and any other value the third, including the
   field's absent-key default of −1. The flag is never cleared during the game. Infiltration itself belongs to the special-abilities specs.
   - Backing: `measurable` (scenario T6)
   - Confidence: high (the requirement); medium (the infiltration conditions and the choice of
     flag; T6 runs 1 to 8 test them)

7. **Superweapons off.** When the game's superweapon setting is off (S-0412), a building type whose
   first superweapon field names a superweapon that may be disabled from the setup screen (the
   superweapon's disableable flag, Import mapping) is refused, unless the building type is on the
   tech-lab list. The second superweapon field plays no part.
   - Backing: `measurable` (scenario T7)
   - Confidence: high

8. **Prerequisites.** Every entry of a type's prerequisite list must be met:
   - (a) A building type entry is met when the house's present count of that type is above 0.
     Buildings in their construction state count; so do captured buildings for their new owner and
     buildings switched off or damaged. Buildings of a type with the don't-score flag never count, so
     such a type never meets a prerequisite.
   - (b) A generic group entry is met when the present count of any type of that group is above 0.
     The refinery group is also met when the house's present count of the refinery-alternative
     vehicle type (Import mapping) is above 0. If that vehicle type has the don't-score flag, its
     present count is never lowered once a vehicle of it has been on the map, so the group stays met
     after the last such vehicle is destroyed (`quirks.unscored_vehicle_stays_counted`, S-0414
     claim 4; with the switch off such a vehicle never counts and never meets the group).
   - (c) A building type entry that is an upgrade type (S-0406, Terms) is met in another way. Of the
     house's buildings, of any type, that are on the map, switched on, not being sold and not queued to be sold, only the one the house acquired most recently decides: the entry is met
     when the upgrade is among the upgrades that building holds (up to three), whatever the other
     buildings hold (`quirks.upgrade_prerequisite_last_building`). An upgrade never enters play as a
     building, so its present count is always 0.
   - (d) A name in the list that is neither a generic group word nor a building type registered at the
     time the type is read (S-0004) is dropped when the list is read; vehicle, infantry and aircraft
     names are dropped too. The importer warns.
   An empty list is met.
   - Backing: `measurable` (scenario T8)
   - Confidence: high (a, b, d's dropping, the don't-score vehicle count); medium (c: that only
     one building decides is tested by T8 (e) for two buildings only; which building counts as the most recently acquired after a capture, a deploy or
     a transfer of ownership is not verified, Open question 3)

9. **Build limit.** Let *L* be the type's build limit.
   - *L* > 0: the answer is at limit when the owned count is *L* or more, allowed otherwise. Exception:
     when the owned count is *L* or more and one of the house's lines has an item of this type in
     production, the answer is allowed (for an infantry type only when the owned count is exactly
     *L*; `quirks.infantry_limit_exception_exact`). For an infantry type with the vehicle-thief flag (glossary, *vehicle thief*), the owned
     count also includes every vehicle of the house that holds a thief of that type after taking it
     over. The thief stops existing as an infantry object when it takes the vehicle over, so one
     take-over counts once, through the vehicle.
   - *L* < 0: refused when the ever-built count is |*L*| or more; allowed otherwise. For
     *L* = −2³¹ the answer is always refused (Terms, *Build limit*).
   - *L* = 0: refused.
   The absent-key default is 2³¹ − 1, in effect no limit. The exception means that an item in
   production does not make its own type unbuildable, although its object is already owned (Terms).
   - Backing: `measurable` (scenario T10, parts a to f and i)
   - Confidence: high (the three cases, the exception); medium (the vehicle-thief count and that a
     take-over counts once, the infantry exactness, the most negative limit)

10. **What counts as ever built.** The ever-built count of a type rises by one when one of the
    house's factories releases an object of that type (a vehicle, infantry or aircraft leaving its
    factory, S-0404 claim 7). A building put on the map by a place order does **not** raise it, so a
    negative build limit other than −2³¹ never takes effect on a building type for a player (−2³¹
    refuses whatever the count, claim 9)
    (`quirks.placed_buildings_not_ever_built`). Objects that come from starting units, crates, deploying
    (S-0415), packing up, capture or triggers never raise it.
    - Backing: `measurable` (scenario T10, parts b, c, g and h)
    - Confidence: medium

### Queueing, orders and countries

11. **Countries come from the factory.** No condition compares the house's own country with a type's
    owner countries. The owner countries matter only through claim 1 (b): the house needs a factory
    building whose type shares an owner country with the type. A house that captures a factory
    building of another country group therefore gets that group's types of the factory's kind,
    subject to the other conditions (country lists, prerequisites, tech level).
    - Backing: `measurable` (scenario T2)
    - Confidence: high

12. **When the offer is worked out again.** The offer of a house (the taken-up set of claim 1 (c) and
    with it which types are offered) is worked out again, a *recheck*, in the first house update
    (S-0203) after one of the events below, and after no other event. The list is closed for our
    engine. Which house update that is follows from the tick order of S-0203, with these observable
    results:
    - an event of the turn pass (a deploy, a capture, an infiltration, a trigger action) changes the
      offer in the house update of the same tick.
    - a placed building changes the offer in the next tick's house update, because place and sell
      orders are applied after the house updates of their tick (S-0203, claim 7).
    - a suspension or resumption of event (k) caused by a change of the house's low-power state,
      which happens inside its house update, is taken up by the recheck of that same house update.
    - a cancellation order issued because of a recheck is applied at the end of that tick (S-0404,
      claim 9). If its application causes an event of this claim, that event is taken up by another
      recheck in the house's next house update; a superweapon found lost at a recheck (event (k))
      is likewise taken up by another recheck in the next house update.

    A recheck works out the whole taken-up set again from the state at that moment, for every type,
    not only for the types that the triggering event touched. So it also applies every change that
    earlier non-events (below) left pending: between rechecks the set can be stale, and whichever
    event comes next makes the stale change visible. An implementation that, at a recheck, looks
    again only at the types touched by the triggering event does not meet this claim (T8 (f) and
    (g); T14 runs 6, 8, 9 and 14). Therefore the list of events can be seen in both directions while
    the set is stale: a missing event delays the appearance or disappearance of an icon until a
    later event, and an extra event brings it forward. Both are differences from the original.

    | Event, for the house concerned | Notes |
    |---|---|
    | (a) A building is put on the map | placement, deploy, capture (for the new owner), a trigger action that changes a building's owner, a building present at game start |
    | (b) A building leaves the map or changes owner | sold, destroyed, packed up, captured (for the old owner), removed by a trigger action; a sale's removal, not the start of the sale |
    | (c) A building is switched on or off | S-0405 says when a building is switched on |
    | (d) A building gains an upgrade | S-0406 |
    | (e) A building starts or stops *being erased* (glossary) | for the building's owner; whether a building being erased counts as switched off is not known (S-0405), so no effect on the answer is stated, only the recheck |
    | (f) A vehicle, infantry or aircraft is released by a factory | S-0404 claim 7; claim 10 |
    | (g) An object of a type that is at limit for its house stops existing (destroyed, or removed from the game in another way, including the object of an item whose production is cancelled, S-0404) | for its owner; its type's tech-tree answer (claim 2) is judged just before the object stops existing, the object itself counted and claim 9's exception for an item in production left out; only the answer at limit is an event |
    | (h) A spy of the house infiltrates a building | claim 6 |
    | (i) A sale ends without removing the building | a rule of our engine; no such sale is known in the original |
    | (j) The house launches a superweapon of the missile kind | the kind whose launch sends a missile to the chosen target cell (Import mapping, superweapon kind); the launch of a superweapon of any other kind is no event |
    | (k) One of the house's superweapons is suspended, resumed or lost | suspended and resumed: its charge is held or released (S-0405 claim 15; the superweapon specs say when); lost: the house no longer has it; the timing inside the house update is given above the table |

    These are *not* events, in our engine as in the original (each with the switch that would make
    it one; the switches default to the original):
    - the start of a sale. A building being sold stops counting for the upgrade prerequisite of claim
      8 (c), as a qualifying factory and as a factory of claim 1 (b), so the start can make a
      condition hold as well as fail, yet it calls for no recheck: a type unlocked that way appears
      only at the removal, event (b) (T14 run 2), and a type whose only factory building is being
      sold stays offered until the removal (T14 run 7) (`quirks.sale_start_no_recheck`),
    - for an infantry, vehicle or aircraft object, any of the following four, although each can
      change a present or owned count and so an input of claims 8 and 9 (for example the refinery
      group through the refinery-alternative vehicle, claim 8 b); a type whose condition they make
      hold or fail changes its offer only at the next recheck caused by an event of the table
      (`quirks.unit_changes_no_recheck` makes these four events, and only these):
      - (l) the object enters the map in any way other than (f), for example unloading from a
        transport, a crate or a trigger action (T14 run 6),
      - (m) the object leaves the map without stopping to exist, for example by entering a transport
        or a building (T14 run 8),
      - (n) the object changes owner, for both houses (T14 run 10),
      - (o) the object stops existing while its type is not at limit for its house, judged as in
        event (g) (T14 run 14).

    A trigger action that changes a type's tech level exists in the original and calls for a recheck
    of every house. It is not among the trigger actions of S-0011's subset, and if a later revision
    adds it, it is an event.
    - Backing: `measurable`. Each scenario looks for an icon that appears or disappears, which only
      a recheck can cause (claim 1; S-0411 claim 2). The darkened state is not used: it follows the
      current counts in every tick, whatever the event. (a) by T1 (placement) and T2 (capture); (b),
      (c), (d) and (f) by T14 runs 1 to 5; (h) by T6; (e), (g), (j) and (k), whose effect on the
      answer is nil or unknown, from a stale starting state, by T14 runs 12, 9 (with the cancelled
      item), 11 and 13; the non-events by T14 runs 2 and 7 (the start of a sale), 6 (l), 8 (m), 10 (n) and 14 (o); the
      whole-set rule by T8 (f) and (g) and T14 runs 6, 8, 9 and 14. The runs from a stale starting
      state depend on T14 run 6 or 8 showing that the original leaves the offer stale after a
      non-event; if it does not, those runs are void and this claim is revised. `unmeasured`: (i),
      because no such sale is known in the original, so it is a rule of our engine with nothing to
      observe; and the further recheck following an event caused by a recheck (a cancellation
      applied at the tick boundary, a superweapon found lost), because seeing it needs a non-event inside the single tick between
      the two house updates, which a scenario cannot arrange reliably (the first recheck already
      shows every earlier change).
    - Confidence: high: the timing, and events (a) to (d), (f) and (h). Medium: events (e), (g)
      and (j), the whole-set rule and the non-events. Low: event (k), since when a superweapon is
      suspended or resumed belongs to the superweapon specs. Medium: the completeness of the list,
      since the original's own list may hold more events (Open question 1).

13. **Build orders follow the offer.** In the original the offer is made by the local player's
    interface: a build order to start or queue an item is issued only for an offered type that is
    not at limit; a request to resume its held item is exempt from that issue-time limit test
    (S-0411 claim 4). In both cases
    the build order itself checks only the factory conditions and the build limit when it is applied
    (S-0404, claim 3). In our engine the offer of claims 1 to 12 is worked out for every player
    house, AI houses included. Every build order requires an offered type (in its house's taken-up
    set, claim 1) when issued. An order to start or queue an item also requires that the type is
    not at limit (claim 9); an order requesting resume of its held item is exempt from that test,
    and from the busy building-line test of S-0411 claim 4. Resume remains subject to S-0404
    claim 3's factory and build-limit application tests. The application tests for starting or
    queuing are also S-0404 claim 3's and claim 16's. A
    type that is still in the taken-up set although claims 5 to 8 have come to refuse it since the
    last recheck (only through a non-event of claim 12) can therefore be ordered and produced, as
    in the original (`quirks.stale_offer_orderable`). When a recheck removes a type from the
    taken-up set, the cancel and cancel-all orders required by S-0404 claim 9 are issued in that
    house update and applied at the end of the same tick, for every player house. S-0411 claim 13
    owns their order when several types are removed. This follows the AI-house ruling (glossary,
    *AI house*).
    For AI designers: in the original, the prerequisite condition (claim 8) and the build limit
    play no part in what a computer-controlled house is offered, and such a house's production is
    never cancelled when a prerequisite is lost.
    - Backing: `measurable` (scenario T11, including the order for a type refused since the last
      recheck; S-0411 S4 b2 for held-item resume at and above the infantry limit; S-0404 P9 for
      cancellation issue and application timing) for the local player's behaviour; the rule for our AI houses is `unmeasured`, a rule
      of our engine; the sentence on the original's
      computer-controlled houses is `unmeasured` too: a scenario cannot force a bystander computer
      player to try a given type (its choices come from its own build logic, and the glossary
      keeps it away from the observed area), so the skip could not be told from the computer
      player simply never choosing that type. It is a note for AI designers and decides no rule of
      this spec.
    - Confidence: high (the local player); medium (that a type refused since the last recheck can
      be ordered); low (the original's computer-controlled houses, not observed: see Backing)

14. **Secret labs at game start.** During the game-start setup, under the conditions of the table
    below, every secret lab on the map, whoever owns it, is given the secret type, or none, that the
    table gives it. Let *N* be the total
    length of the game-wide secret infantry, secret vehicle and secret building lists, concatenated
    in that order, and *n* the number of secret labs on the map. The labs are numbered *k* = 0, 1, …
    in the order they were created (for the map's buildings, the map's order, S-0009). A lab's
    *fixed secret type* is the first that its type gives of a fixed secret infantry, vehicle and
    building type, in that order.

    | Condition | Draws from the scenario random stream | Secret type of lab *k* |
    |---|---|---|
    | *n* = 0, *N* = 0 or *n* > *N* | none | its fixed secret type if it has one; none from the lists |
    | 1 ≤ *n* ≤ *N* | one ranged draw *r*ₖ over 0 … *N* − 1 − *k* for every lab, a lab with a fixed secret type included (F4) | its fixed secret type if it has one; otherwise entry *r*ₖ of the concatenated list, counted from 0 |

    On the random-state recorder the labs' draws come after the draws taken while the starting units
    are placed. Because the range shrinks but the entry is taken by position, two labs can get the
    same type and the last entries can never be drawn by later labs
    (`quirks.secret_lab_pick_by_position`). The range of the last draw is 0 … 0 when *N* − *k* = 1, and
    a ranged draw over a single value consumes no raw value (S-0202 claim 4), so that draw leaves the
    stream state unchanged.
    - Backing: `measurable` (scenario T9, with the random-state recorder of the measurement setup)
    - Confidence: medium

15. **Captured buildings grant tech.** A captured building counts for its new owner from the tick of
    the capture, and no longer for its old owner: as a prerequisite (claim 8), as a factory (claims 1
    and 11), as an override building (claim 3) and, for a secret lab, with its secret type (claim 4).
    Capturing does not grant stolen tech (claim 6 needs a spy) and does not change either house's
    ever-built counts.
    - Backing: `measurable` (scenarios T2, T9 and T13; the ever-built counts by T10 (h))
    - Confidence: high

16. **Queue-time limit.** When a type is queued behind an item (S-0404, claim 3: the line holds an
    item or a queue, so the line exists), the build limit is checked again with the line's item and
    queued copies added. The line is the house's line of the type's kind (for a vehicle type, the
    naval or the ground line by its naval flag). For *L* > 0 the order is refused when the owned
    count plus the number of the type's copies in the line (the item and the queue) is *L* or more,
    and for *L* ≤ 0 when the ever-built count plus that number is |*L*| or more (always for
    *L* = −2³¹, F3). The item in
    production is thus counted twice: once in the owned count (its object exists) and once in the
    line (`quirks.queue_limit_counts_item_twice`). An aircraft type that must return to a dock
    (Import mapping) is judged differently at queue time, and its own build limit plays no part
    there (`quirks.dock_bound_aircraft_ignores_limit`). Let *D* be the sum of
    - the house's present count of every type on the game-wide pad aircraft list (S-0404, Import
      mapping), and
    - the number of copies, in the house's aircraft line (the item and the queue), of every type on
      that list (the queued type's own copies count only if the type is on the list),
    and let *T* be the total number of docks of the house's dock buildings (each building with the
    dock flag adds its number of docks). The order is refused when *D* is *T* or more (F3). A type
    that is not on the pad list adds neither its objects nor its copies to *D*. So a house can
    queue copies of a dock-bound aircraft type beyond its build limit, while an item of the type is
    in production (claim 9's exception keeps the type orderable then). Acceptance into the queue
    does not guarantee production: when a queued copy would start, S-0404 claim 10 tests factory
    eligibility and claim 9's build-limit answer with the production allowance, though it does
    not repeat this claim's queue-time test or the full tech-tree test. With no other item of
    that type in production, a copy at its positive limit therefore cannot start.
    - Backing: `measurable` (scenario T12, runs 1 to 6)
    - Confidence: medium (queue tests and the application-time limit on a queued start; T12 run 6)

## Formulas

> **F1. Country-list membership.** Let *i* be the registration index of the house's country and *K*
> the set of registration indices of the known countries a list names. The house's country is in the
> list when *i* mod 32 = *j* mod 32 for some *j* in *K*. Integers, exact.

> **F2. Build-limit answer** (claim 9). With owned count *O*, ever-built count *E*, *P* = 1 when an
> item of the type is in production in one of the house's lines and 0 otherwise:
> *L* > 0: allowed when *O* < *L*, or *P* = 1 and (*O* = *L*, or *O* > *L* for a non-infantry type),
> at limit otherwise. *L* ≤ 0: refused when *L* = −2³¹ or *E* ≥ −*L*, allowed otherwise (so *L* = 0
> is always refused). Signed 32-bit integers (*L* from −2³¹ to 2³¹ − 1; −*L* is used only for
> *L* > −2³¹, where it is exact); comparisons only, no rounding. With `quirks.infantry_limit_exception_exact`
> off, an infantry type follows the rule for the other types: for *L* > 0 it is allowed when *O* < *L*
> or *P* = 1, at limit otherwise.

> **F3. Queue-time refusal** (claim 16). With *Q* the number of copies of the type in the line
> (item and queue): *L* > 0: refused when *O* + *Q* ≥ *L*. *L* ≤ 0: refused when *L* = −2³¹ or
> *E* + *Q* ≥ −*L*.
> Dock-bound aircraft type: with *D* the sum defined in claim 16 and *T* the total number of docks,
> refused when *D* ≥ *T*; *L*, *O* and *E* play no part (with
> `quirks.dock_bound_aircraft_ignores_limit` off the order is refused when the dock test or the
> *L* test above refuses). All quantities and intermediate sums are signed 32-bit integers; an
> integer result outside the signed 32-bit range wraps modulo 2³², as in S-0205 claim 1. −*L* is
> as in F2; comparisons only, no rounding.

> **F4. Secret lab draw** (claim 14). Lab *k* = 0 … *n* − 1 (*n* ≤ *N*): *r*ₖ = one ranged draw over
> 0 … *N* − 1 − *k* (S-0202, claim 4, the range rule), secret type = entry *r*ₖ of the concatenated list.
> Integers, exact; the arithmetic inside the ranged draw is S-0202's.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Tech level absent-key default | 255 | level | claim 5 |
| Tech level meaning "never" | −1 | level | claim 5 |
| Build limit absent-key default | 2³¹ − 1 | objects | claim 9 |
| Country-list index modulus | 32 | registration indices | F1 |
| Stolen-tech side values | 0, 1, other | side value (an enumeration, no unit) | claim 6; the values of the base-planning-side field |
| Generic groups | 6 | groups | power, factory, barracks, radar, tech, refinery |

## State and ordering

| State | Event | Result |
|---|---|---|
| Building not on the map, or owned by another house | Building put on the map (placement, deploy, capture) | present count up; recheck needed, claim 12 (a) |
| Building on the map, owned by the house | Building leaves the map or changes owner | present count down for the old owner; recheck needed, claim 12 (b) |
| any | Item starts | owned count up (the item's object); queue-time limit counts it twice (claim 16) |
| any | Unit released by a factory | ever-built count up (claim 10); recheck needed, claim 12 (f) |
| any | Spy infiltrates a tech-lab building | stolen-tech flag set (claim 6); recheck needed, claim 12 (h) |
| Type offered or not offered | Claim 1 (a) or (b) starts or stops holding without an event of claim 12 (the start of a sale, or a non-event (l) to (o)) | the offer of the type is unchanged until the next recheck, whatever event causes it (claims 1 and 12) |
| Type offered and in production | A recheck removes the type from the taken-up set | S-0404 claim 9 orders are issued in that house update and applied at the end of that tick |

Observable ordering: a type unlocked by a placed building appears in the house update of the tick
after the placement order is applied, while the building is still in its construction state.

## Edge cases and quirks

- `quirks.unscored_vehicle_stays_counted` (default original; owned by S-0414, claim 4). It is the
  single switch for the present count of vehicle types as well: claim 8 (b) reads the count it
  keeps. This spec adds no switch of its own.
- `quirks.upgrade_prerequisite_last_building` (default original): claim 8 (c) examines only the most
  recently acquired eligible building. Off: the entry is met when any building of the house that is
  on the map, switched on, not being sold and not queued to be sold holds the upgrade.
- `quirks.placed_buildings_not_ever_built` (default original): claim 10. Off: a building put on the map by
  a place order raises its type's ever-built count, so a negative build limit on a building type
  works.
- `quirks.secret_lab_pick_by_position` (default original): claim 14. Off: lab *k* gets the entry that
  stands at position *r*ₖ among the entries not yet given (drawing without replacement), so no two
  labs share a secret type. The number and range of the draws are the same. A lab with a fixed secret
  type still takes its draw, and the entry it draws counts as given (it is not drawn again), because
  the range of lab *k* is 0 … *N* − 1 − *k*, which fits exactly the *N* − *k* entries left after *k* are given.
- `quirks.unit_changes_no_recheck` (default original): claim 12. An infantry, vehicle or aircraft
  object calls for no recheck when it enters the map other than by release from a factory (l),
  leaves the map without stopping to exist (m), changes owner (n), or stops existing while its type
  is not at limit (o). Off: exactly these four, (l) to (o) of claim 12, become events of the table,
  for every house whose objects they concern (both houses, in case (n)); nothing else changes. In
  particular the start of a sale stays a non-event, which `quirks.sale_start_no_recheck` governs.
- `quirks.sale_start_no_recheck` (default original): claim 12. The start of a sale calls for no
  recheck, so a type that the start of the sale unlocks appears, and a type whose only factory
  building is being sold disappears, only at the removal. Off: the start of a sale is an event of
  claim 12 for the building's owner, so both happen in the first house update after the sale
  starts.
- `quirks.stale_offer_orderable` (default original): claim 13. A type that stays in the taken-up set
  after claims 5 to 8 have come to refuse it (only through a non-event of claim 12) can be ordered
  until the next recheck, since a build order tests only the offer and the build limit. Off: a
  build order is also refused when the type's tech-tree answer, worked out when the order is
  issued, is refused.
- `quirks.queue_limit_counts_item_twice` (default original): claim 16. Off: the item in production is
  counted once.
- `quirks.dock_bound_aircraft_ignores_limit` (default original): claim 16. Off: a dock-bound aircraft
  type is refused at queue time when either the dock test (*D* ≥ *T*) or the build-limit test of
  any other type (F3: *O* + *Q* ≥ *L*; for *L* ≤ 0, *L* = −2³¹ or *E* + *Q* ≥ −*L*) refuses it.
- `quirks.infantry_limit_exception_exact` (default original): claim 9. Off: an infantry type is allowed
  while an item of it is in production whenever the owned count is *L* or more, as for the other
  kinds, not only when it is exactly *L*.
- `quirks.country_index_wraps` (default original): F1. Countries whose registration indices are equal
  modulo 32 (for example 1 and 33) cannot be told apart by a country list: naming one counts as
  naming the other. Off: a country is in a list only when the list names it. The importer warns when
  more than 32 countries are registered.
- **Don't-score buildings** never meet a prerequisite and are never counted for limits (claims 8
  and 9). No switch; the importer warns when a prerequisite list names a don't-score building type.
- **Dropped prerequisite names** (claim 8 d): a misspelt name makes the condition easier, not
  harder. The importer warns.

## Import mapping

| Key (file, section) | Concept | Form | Absent |
|---|---|---|---|
| `Prerequisite` (rules, type) | prerequisite list (claim 8) | list of building type names and group words | empty |
| `PrerequisiteOverride` (rules, type) | override list (claim 3) | list of building type names (a group word is bad data, claim 3) | empty |
| `POWER`, `FACTORY`, `BARRACKS`, `RADAR`, `TECH`, `PROC` (words in the two lists above, compared ignoring case) | the six generic groups (claim 8 b) | word | — |
| `PrerequisitePower`, `PrerequisiteFactory`, `PrerequisiteBarracks`, `PrerequisiteRadar`, `PrerequisiteTech`, `PrerequisiteProc` (rules, `General`) | members of the six groups | list of building type names | empty |
| `PrerequisiteProcAlternate` (rules, `General`) | refinery-alternative vehicle type (claim 8 b) | vehicle type name | none |
| `TechLevel` (rules, type) | tech level (claim 5) | integer | 255 |
| `RequiredHouses`, `ForbiddenHouses` (rules, type) | required and forbidden country lists (claim 5) | list of country names | no restriction |
| `RequiresStolenAlliedTech`, `RequiresStolenSovietTech`, `RequiresStolenThirdTech` (rules, type) | needs the first, second, third stolen-tech flag (claim 6) | boolean | no |
| `AIBasePlanningSide` (rules, building type) | which stolen-tech flag an infiltration gives (claim 6) | integer | −1 |
| `BuildTech` (rules, `AI`) | tech-lab list (claims 6 and 7) | list of building type names | empty |
| `SuperWeapon`, `SuperWeapon2` (rules, building type) | first and second superweapon (claim 7) | superweapon name | none |
| `DisableableFromShell` (rules, superweapon) | disableable flag (claim 7) | boolean | no |
| `Type` (rules, superweapon) | superweapon kind; only the missile kind matters here (claim 12 (j)); its word waits for the allowed-key list and the superweapon specs | word | none (no kind, so no event) |
| `BuildLimit` (rules, type) | build limit *L* (claims 9, 10, 16) | integer | 2³¹ − 1 |
| `Owner` (rules, type) | owner countries (claims 1 and 11) | list of country names | none (never offered) |
| `Factory` (rules, building type) | factory kind (claim 1) | type kind | none |
| `Naval` (rules, type) | naval kind (claim 1) | boolean | no |
| `DontScore` (rules, type) | don't-score flag (Terms, claims 8 and 9) | boolean | no |
| `Insignificant` (rules, type) | insignificant flag (Terms) | boolean | no |
| `VehicleThief` (rules, infantry type) | vehicle-thief flag (claim 9) | boolean | no |
| `SecretLab` (rules, building type) | secret-lab flag (claims 4 and 14) | boolean | no |
| `SecretInfantry`, `SecretUnits`, `SecretBuildings` (rules, `General`) | the three secret lists (claim 14) | lists of type names | empty |
| `SecretInfantry`, `SecretUnit`, `SecretBuilding` (rules, building type) | a lab's fixed secret type (claim 14) | type name | none |
| `AirportBound` (rules, aircraft type) | dock-bound aircraft (claim 16) | boolean | no |
| `PadAircraft` (rules, `General`) | the pad aircraft types counted in claim 16 (S-0404) | list of type names | empty |
| `NumberOfDocks` (rules, building type) | docks a building adds to the house total (claim 16) | integer | 0 |
| `Helipad` (rules, building type) | the building counts its docks for claim 16 | boolean | no |

The owner-country and country-list values are compared with the country registry of S-0004. The
list keys above take the text limit of S-0003 (Import mapping rows C3 and C4): items beyond it are
lost.

## Measurement scenarios

Measurement setup (glossary); a tiny purpose-made skirmish map, one human house (the local player,
whose build icons are observed) and one bystander computer player (glossary), named test rules
files in the map's override sections, scripted input, test triggers to give, destroy and capture
buildings, and a recorder logging the sidebar's icon list and darkened state per tick. The
observed area of every scenario is the human house's base and the buildings of the bystander that
the scenario names (captured, infiltrated or spied on): these are the bystander objects the
scenario uses, and the glossary's void rule applies to every other object of the bystander. The
bystander's country belongs to a different side than the human house's, and it is the "enemy"
of the scenarios.

**T1. Offer timing and the switched-on condition.**
- Setup: a test type needing a test prerequisite building; place the prerequisite (record the tick
  the place order is applied and the tick its construction state ends); second run with the house's
  only barracks switched off before the prerequisite is placed, then switched on.
- Observable: the tick the icon appears; second run: whether it appears before the barracks is
  switched on.
- Expected: the icon appears one tick after the placement, before the construction state ends,
  second run: it appears only after the switch-on.
- Falsified if: the icon waits for the end of the construction state, or appears while the only
  factory is switched off.

**T2. Captured factories and owner countries.**
- Runs 1 and 2 (one game). Setup: the human house, whose country belongs to one side, captures (test trigger) a barracks
  and a vehicle factory building owned by the bystander, whose country belongs to another side; the
  two captured buildings are named bystander objects of the observed area. Test types with owner
  countries of the bystander's group only, one of them with a required list naming an enemy
  country.
- Observable: icons.
- Expected: the enemy-group types appear (run 1), except the one with the required list (run 2).
- Falsified if: capture grants nothing (run 1), or the required-list type appears (run 2).
- Run 3 (a failed factory condition with a qualifying factory present; claim 1 (c)): a test naval
  vehicle type (naval flag, owner countries naming the human house's country, no other unmet
  condition); the human house has a switched-on, non-naval vehicle factory building and no naval
  factory building. Observable: the icon at every tick of the run. Expected: the icon never
  appears. Falsified if: it appears at any tick.
- Run 4 (a satisfied factory condition with no qualifying factory): the type of run 3 with owner
  countries naming both the human house's country and the bystander's; the human house's vehicle
  factory building is switched off; it captures (test trigger) a switched-off naval factory
  building of the bystander (a named bystander object). Then the vehicle factory building is
  switched on. Expected: no icon after the capture; the icon appears in the first house update
  after the switch-on (claim 12), and stays when the vehicle factory building is switched off
  again. Falsified if: the icon appears after the capture while no factory is switched on, does not
  appear after the switch-on, or is gone after the switch-off.

**T3. Override list.**
- Setup: a test type with tech level above the house tech level and an unmet prerequisite, and an
  override list naming a test building; give the house that building.
- Observable: icon.
- Expected: offered once the override building is on the map; gone when it is destroyed.
- Falsified if: the type stays unoffered, or is still offered after the override building is
  destroyed.
- Second run (precedence, claim 2): as the setup above, with the test type's build limit 1; give the override
  building, then one object of the type by test trigger. Expected: the icon is present and
  darkened (lifted, at limit); after the override building is destroyed the icon is gone (not
  lifted, refused takes precedence over at limit). Falsified if: the icon is still shown, darkened
  or not, after the override building is destroyed, or is not darkened while the override building
  stands.

**T4. Tech level.**
- Setup: a house tech level below 255; test types with tech levels −1, the house tech level, one
  above, and none (key absent).
- Observable: which of the four icons are present once the house has a barracks.
- Expected: only the type at the house tech level is offered.
- Falsified if: the absent-key type or the −1 type is offered, the type at the house tech level
  is not offered, or the type above it is offered.

**T5. Country lists.**
- Setup: test types with a required list naming the house's country, one naming another country,
  one naming only an unknown name; a type with a forbidden list naming the house's country; a type with
  a forbidden list naming only unknown names.
- Observable: which icons are present.
- Expected, in that order: offered, not offered, not offered (required lists), not offered, offered
  (forbidden lists).
- Falsified if: any differs.
- Run 2 (index wrap, `quirks.country_index_wraps`): test rules register enough countries that a
  test country has registration index *i* + 32, where *i* is the registration index of the human
  house's country; a test type whose required list names only that other country. Expected with
  the switch on: offered; with the switch off: not offered. Falsified if: the outcome differs
  from the switch setting.
- Run 3 (empty and absent lists, claim 5 (c)): three test types: one with no country-list keys,
  one whose required list is given empty, one whose forbidden list is given empty. Expected: all
  three offered to the human house. Falsified if: any is not offered.

**T6. Stolen tech.**
- Setup: three test types, needing the first, the second and the third stolen-tech flag
  respectively. Tech-lab buildings owned by the bystander (named bystander objects of the observed
  area), each of a test type that is not a radar building and has no power output unless stated, and a spy
  of the human house that infiltrates one building per run. Runs 1 to 8, each from a fresh game:
  1. the building's base-planning-side field is 0 (on the tech-lab list),
  2. it is 1 (on the list),
  3. the field is absent (on the list),
  4. the field is 5 (on the list),
  5. as run 2, but the building type is a radar building (it has the radar flag),
  6. as run 2, but the building type has a power output above 0,
  7. as run 2, but the building type is not on the tech-lab list,
  8. as run 2, with a second infiltration of the same building later in the same game.
- Observable: the three icons before and after the infiltration tick, and at the end of the game.
- Expected: after the infiltration only these icons appear: run 1 the first-flag type, run 2 the
  second-flag type, runs 3 and 4 the third-flag type; none in runs 5 to 7. The flag stays set until
  the end of the game, also in run 8.
- Falsified if: a different type than listed appears in runs 1 to 4, any type appears in runs 5 to
  7, or an icon is gone before the end of the game.

**T7. Superweapons off.**
- Setup: superweapon option off; a test building with a disableable superweapon; one with a
  non-disableable superweapon; one on the tech-lab list with a disableable superweapon; one with the
  disableable superweapon only in its second field.
- Observable: which icons are present.
- Expected: only the first is not offered.
- Falsified if: any of the other three is not offered.

**T8. Prerequisites and groups.**
- Setup: a test type needing the power group and a named building; another needing the refinery
  group; give (a) only a group member, (b) both, (c) for the second type only a refinery-alternative
  vehicle, released from the house's vehicle factory (claim 12 (f); a vehicle given by test trigger
  calls for no recheck, T14 run 6); (d) a test type whose prerequisite is a don't-score building; (e) an upgrade prerequisite
  with two qualifying buildings, the upgrade on the older one only; (f) as (c) with the
  refinery-alternative vehicle type given the don't-score flag in a test rules file, the vehicle
  then destroyed; (g) as (f) with the vehicle entering a transport instead; (h) a test type
  whose prerequisite list names only a vehicle type and a name that is not registered; (i) a
  test type needing one named building, which is then switched off and, in another run, damaged
  to a low health above 0.
- Observable: the test types' icons in each case.
- Expected: (a) no, (b) yes, (c) yes, (d) never, (e) not offered (quirk), (f) still yes after the
  vehicle is destroyed (quirk), (g) still yes while it sits in the transport; in (f) and (g) also
  after a later event of claim 12 (a test building put on the map), since the frozen offer alone
  (claim 1) would keep the icon until then, (h) offered (both names are dropped, and the
  empty list is met), (i) still offered in both runs.
- Falsified if: any differs.

**T9. Secret labs.**
- Setup: a map with three secret labs whose types name no fixed secret type, test secret lists of
  total length 3; the recorder logs the scenario stream state before and after the assignment.
- Observable: each lab's secret type (the icon, after the human house captures the lab); the stream
  state before and after the assignment; the result of each draw.
- Run 1 (the assignment). Expected: three ranged draws over 0 … 2, 0 … 1 and 0 … 0, the last
  consuming no raw value (S-0202 claim 4), so the stream state advances by the raw values of the
  first two only; the types by position (duplicates possible across seeds); the captured lab's
  secret type offered from the capture. In every run the recorder shows the lab draws after the
  draws taken while the starting units are placed.
- Run 2 (a fixed type): the first-created lab (*k* = 0, so its draw is over 0 … 2 and takes a raw value) is one whose type names a fixed secret infantry type and also a fixed
  secret building type (the infantry kind is listed first of the three kinds: infantry, vehicle,
  building); the map has three labs and secret lists of total length 3, as in run 1. Expected: that
  lab gives the fixed infantry type, and the recorder shows three draws as in run 1, the first of
  them over 0 … 2 and consuming a raw value, so the stream state advances as in run 1. If the
  lab with a fixed type took no draw, the later labs' draws would be over different ranges and the
  stream state would differ from run 1.
- Run 3 (loss of the lab, claim 4): as run 1, the human house captures one lab and its secret type
  is offered; the lab is then destroyed (test trigger). Expected: the icon is gone in the first
  house update after the destruction (claim 12), unless another condition lifts the type.
- Run 4 (more labs than entries, and no entries): four labs and test secret lists of total
  length 3; then a second game with four labs and empty secret lists. Expected: in both games the
  stream state is the same before and after the assignment (no draw is taken) and no lab gives a
  type from the lists.
- Run 5: as the first game of run 4, with one of the labs naming a fixed secret infantry type.
  Expected: that lab gives its fixed type, the others give none, and the stream state does not
  change.
- Falsified if: the draws or the assignment differ in run 1, the fixed building type is given
  instead of the infantry type, or the draws or the stream state of run 2 differ from run 1, a lab draw
  comes before a draw of the starting-unit placement, the icon of run 3 survives the destruction of
  the lab, the stream advances in run 4 or 5, a lab gives a type from the lists in run 4, or the lab
  with the fixed type gives none in run 5.

**T10. Build limits.**
- Setup: (a) a test vehicle type with *L* = 1: start one, observe the icon while it is in production
  and after release; in the same game, a test infantry type with *L* = 1 of which the house owns
  none: start one and observe its icon while it is in production (the owned count is then 1 =
  *L*); (b) a test vehicle type with *L* = −1: build one, lose it; (c) a test building
  type with *L* = −1: place one, sell it; (d) a thief of a test vehicle-thief type takes over an
  enemy vehicle (a named bystander object), the house owning no other object of the thief's type,
  once with the type's *L* = 1 and once, in a second game, with *L* = 2; (e) a test infantry type
  and a test vehicle type, each with *L* = 1: start one of each, then give the house two objects of
  each by test trigger; (f) a test vehicle type with *L* = 0 and one with *L* equal to the most
  negative 32-bit integer, no other condition unmet; (g) a test vehicle type with *L* = −1 of which
  the house is given one by each of: a starting unit, a crate, the packing up of a building into it
  (S-0415), and a test trigger; then a copy is built by the factory; (h) ever-built counts and
  changes of owner: (1) a test building type with *L* = −1, offered by the house's construction
  yard, of which the human house captures (test trigger) a building owned by the bystander (a named
  bystander object); (2) a test vehicle type with *L* = −1 of which the human house builds one
  (the icon goes), then the vehicle is given to the bystander by a test trigger; (3) a test vehicle
  type with *L* = −1 of which the bystander owns one vehicle (a named bystander object) that a test
  trigger gives to the human house; (4) a vehicle of the human house deploys (S-0415) into a
  building of a test building type with *L* = −1, offered by the construction yard; (i) owned-count
  exclusions: a test vehicle type with *L* = 1 and the don't-score flag, and in a second game a test
  vehicle type with *L* = 1 and the insignificant flag; the house builds one (released from the
  factory, no item of the type in production afterwards).
- In (g) and in (h) (2) and (3), after each step a test building is put on the map, so that an event
  of claim 12 follows the step (a given object and a change of owner call for no recheck, claim 12
  (l) and (n), and the frozen offer would hide a change of the answer); the icon is read after the
  first house update that follows. In (h) (1) and (4) the capture and the deploy are themselves
  events (a) of claim 12.
- Observable: icon present and darkened state per tick; in (d) the thief type's icon while the
  thief sits in the vehicle; in (e) both icons while the item of each type is in production
  (the owned count is then 3); in (f) the icons at game start; in (g) and (h) the icon after the
  recheck that follows each step; in (i) the icon after the release.
- Expected: (a) the vehicle icon allowed (not darkened) while in production, at limit after
  release; the infantry icon not darkened while in production (the owned count is exactly *L*, so
  the production exception applies to infantry too); (b) refused after the first is released, even
  after losing it; (c) still offered after placing and selling (quirk); (d) while the thief sits in
  the vehicle, at limit with *L* = 1 (the vehicle counts) and allowed, not darkened, with *L* = 2
  (the take-over counts once); (e) the infantry icon darkened (the owned count is above *L*, so the
  production exception does not apply to infantry), the vehicle icon not darkened (the exception
  applies to vehicles above *L*); (f) neither icon offered; (g) the icon stays offered after every
  given object, and is gone only after the factory release; (h) (1) the building type stays offered
  after the capture, (2) the vehicle type stays not offered after the vehicle has passed to the
  bystander, (3) the vehicle type stays offered after the vehicle has passed to the human house,
  (4) the building type stays offered after the deploy; (i) neither icon darkened while the house
  owns the vehicle (objects of either flag are not in the owned count).
- Falsified if: any differs. In (a), a darkened infantry icon would mean that the production
  exception does not apply to infantry at exactly *L*. In (e), an undarkened infantry icon would
  mean that the infantry exception is not exact and the default of
  `quirks.infantry_limit_exception_exact` is wrong. In (h), any change would mean that a capture, a
  change of owner or a deploy moves an ever-built count (claims 10 and 15). In (i), a darkened icon
  would mean that such objects count towards a positive limit.

**T11. Build order and the offer.**
- Run 1 (at limit): the local player's build order (scripted input) for an offered test vehicle
  type with *L* = 1 of which the house owns one on the map, with no item of it in production.
- Run 2 (refused since the last recheck, `quirks.stale_offer_orderable`): the state of T14 run 8
  after the refinery-alternative vehicle has entered the transport and before any later event: the
  type *Y* is still offered although the refinery group is no longer met. The local player's build
  order (scripted input) for *Y*.
- Observable: whether a line starts or queues the type (the progress shown on the build icon,
  credits).
- Expected: run 1, no production starts; run 2, production of *Y* starts.
- Falsified if: production starts in run 1, or does not start in run 2.

**T12. Queue-time limit.**
- Setup: a test vehicle type with *L* = 2; start one and try to queue a second; second run with
  *L* = 3.
- Observable: the queue count on the icon and the refusal sound.
- Expected: run 1 refused; run 2 accepted, a third queued copy refused.
- Falsified if: run 1 accepts the second copy, or run 2 refuses the second copy or accepts the
  third.
- Run 3 (dock-bound aircraft on the pad list): dock buildings giving *T* = 2 docks in all; two test
  aircraft types, both on the pad aircraft list: *A*, dock-bound with a build limit of 1, and *B*,
  with no limit. The house owns one *B* on the map and queues two copies of *A*.
  Expected: the first copy starts (the line is idle, so the queue-time test is not run); the second,
  queued behind it, is refused (*D* = 1 present *B* + 1 copy in the line = 2 = *T*); with *T* = 3 the second copy is accepted, although *A*'s owned
  count is then 1 and its limit 1, so the build limit plainly plays no part.
  Falsified if: the second copy is accepted with *T* = 2, or refused with *T* = 3, or the first copy
  is refused.
- Run 4 (dock-bound aircraft off the list): as run 3 with *T* = 2 but *A* removed from the pad
  aircraft list (*B* stays). Expected: the second copy of *A* is accepted (*D* = 1 present *B* + 0
  copies of *B* = 1 < 2). Falsified if: it is refused.
- Run 5 (negative limit at queue time, claim 16): a test vehicle type with *L* = −2; one copy is
  released from the factory, a second is started, and a third is queued. Expected: the queued
  copy is refused (*E* = 1 + 1 copy in the line = 2 = |*L*|); with *L* = −3 it is accepted.
  Falsified if: it is accepted with *L* = −2 or refused with *L* = −3.
- Run 6 (the application-time limit on a queued start, claim 16): as run 3 with *T* = 3. Keep the
  first *A* owned, no other item of *A* in production, and a factory with a free dock. Observable:
  the second copy of *A* after the first is released, the queue count and the owned count of *A*.
  Expected: the second copy is accepted into the queue but does not start after release of the
  first, because the owned count is 1 = *L* and the production allowance no longer applies.
  S-0404 claim 10 governs the failed queued start. Falsified if: the second copy starts while
  these conditions hold. A second game with *L* = 2 is the control: the second copy starts and is
  released, leaving two owned *A*. Failure of that control makes the run inconclusive.

**T13. Captured prerequisite and override buildings (claims 3 and 15).**
- Setup: three test types: type 1 needs a test building *X* as a prerequisite; type 2 has a tech
  level above the house tech level and an override list naming a test building *Y*; type 3 needs a
  stolen-tech flag that the house lacks and also has *Y* in its override list. The bystander owns
  *X* and *Y* (named bystander objects of the observed area); the human house owns neither. The
  human house captures both (test trigger), and later *X* and *Y* are destroyed.
- Observable: the icons of the three types at every tick.
- Expected: none before the capture; types 1, 2 and 3 appear in the first house update after the
  capture (the override lifts the stolen-tech condition as well); each disappears after its building
  is destroyed.
- Falsified if: a type appears before the capture or does not appear after it, or is still shown
  after the destruction of its building.

**T14. Rechecks, seen as icons that appear or disappear (claim 12).** Only an icon that appears or
disappears can show that an event called for a recheck; the darkened state cannot (claim 12,
Backing).
- Setup (runs 1 to 4): the human house has a switched-on factory *F* that makes a test infantry type
  *X*, which needs an upgrade type *U* as its only prerequisite. The house has two buildings that
  can hold *U*: *E1*, the older, holding *U*, and *E2*, the newer, without it. *E1* and *E2* are the
  last two buildings the house acquired, after *F* and every other building it owns. By claim 8 (c)
  only *E2* decides, so *X* is not offered. *F* is never affected by what the runs do to *E2*.
- Observable: the tick at which the icon of *X* appears.
- Run 1 (b, destruction): *E2* is destroyed by a test trigger. Expected: the icon appears in the
  first house update after the destruction (*E1* now decides).
- Run 2 (b, sale, and the start of a sale as a non-event): *E2* is sold. While it is being sold it
  is not eligible for claim 8 (c), so *E1* decides. Expected: the icon appears in the first house
  update after the building is removed, and not at the start of the sale.
- Run 3 (c): *E2* is switched off by a means of S-0405 that spares *F* and *E1*. Expected: the icon
  appears in the first house update after the switch-off.
- Run 4 (d): *E2* gains *U* (S-0406). Expected: the icon appears in the first house update after
  the upgrade is gained.
- Run 5 (f): a test vehicle type *Y* needs the refinery group and nothing else, no building of the
  refinery group is on the map, and the house's vehicle factory makes a vehicle of the
  refinery-alternative type *V* (not a don't-score type). The vehicle is released from the
  factory. Expected: the icon of *Y* appears in the first house update after the release.
- Run 6 (a unit entry that is no event, claim 12 (l), `quirks.unit_changes_no_recheck`): as run 5,
  but a *V* is given by a test trigger instead of being built. Expected: the icon of *Y* does not
  appear in the following ticks. It appears in the first house update after a later event of claim
  12 (for example, a test building put on the map). With the switch off it appears in the first
  house update after the entry.
- Run 7 (the start of a sale as a non-event, seen as a disappearance; claim 1's frozen offer): the
  human house's only barracks *B* is switched on and makes a test infantry type *Z* with no unmet
  condition; an unfinished item of *Z* is in production. Give the test type a build time long
  enough to remain unfinished through the sale. *B* is sold, with no other recheck event during
  the sale. Observable: the icon of *Z*, progress, paid credits, refund and cancellation-order
  issue/application ticks, from the sell order through the first house update after *B* is removed.
  Expected: the icon stays during the sale and the item continues due steps and payments, absent
  any independent hold or cancellation (S-0411 claim 14). The item is refunded in the removal
  tick by S-0404 claim 8's permanent-removal rule. The icon is gone in the first house update
  after removal; any cancellation orders issued then apply at that tick's end and give no second
  refund for the already-cancelled item (S-0404 claim 9). With `quirks.sale_start_no_recheck`
  off, the icon disappears in the first house update after the sale starts and its cancellation
  orders apply at that tick's end, refunding an item still present then. Likewise in run 2 the
  icon of *X* appears in the first house update after the sale starts.
- Run 8 (a departure without stopping to exist, claim 12 (m); claim 1's frozen offer): as run 5,
  after *Y* has appeared; the house also owns a test transport that can carry *V*, and *V* enters
  it. Later a test building is put on the map. Observable: the icon of *Y* at every tick.
  Expected: the icon stays while *V* is in the transport and no event of claim 12 has happened; it
  is gone in the first house update after the test building is put on the map. With
  `quirks.unit_changes_no_recheck` off it is gone in the first house update after *V* enters the
  transport.
- Stale starting state for runs 9 to 13: the state of run 6 after *V* has been given and before
  any later event: *Y* is not offered although the refinery group is met. Before *V* is given, the
  house also builds what the run names (each release is event (f), so its recheck comes before the
  stale state begins). Each run then causes one candidate event and nothing else, and later puts a
  test building on the map as a control event. The observable is the icon of *Y* at every tick.
  These runs are valid only once run 6 or run 8 has shown that the original leaves the offer stale
  after a non-event.
- Run 9 (g, and (o) as its control): the house has built one vehicle of a test vehicle type *W* with
  *L* = 1, so *W* is at limit. A test trigger destroys the *W*. Expected: the icon of *Y* appears in
  the first house update after the destruction. Second game, the same with *W*'s *L* = 2 (not at
  limit): the icon of *Y* does not appear until the control event, and appears in the first house
  update after it (with `quirks.unit_changes_no_recheck` off, in the first house update after the
  destruction). Third game (a cancelled item): the house owns no *W* (*L* = 1) but has an item of
  *W* in production, so the owned count is 1 = *L*; the local player cancels the item (scripted
  input). Expected: the icon of *Y* appears in the first house update after the cancel is applied
  (the type is at limit when the production exception is left out, event (g)).
- Run 10 (n, a change of owner): as the first game of run 9, but a test trigger gives the *W* to
  the bystander (it becomes a named bystander object) instead of destroying it. Expected: the icon
  of *Y* does not appear until the control event, and appears in the first house update after it.
  With `quirks.unit_changes_no_recheck` off it appears in the first house update after the change
  of owner.
- Run 11 (j, a superweapon launch): the house has a test building whose first superweapon field
  names a test superweapon of the missile kind with a short charge, ready; the local player
  launches it (scripted input) at a cell far from every object. Expected: the icon of *Y* appears
  in the first house update after the launch. Second game, the same with a test superweapon of
  another kind whose effect creates, moves, damages and removes no object (for example a kind that
  only reveals cells): the icon of *Y* does not appear until the control event.
- Run 12 (e, the start and the end of erasing): the house owns a test building *G* that is no
  factory and is named in no list of the test types. A unit of the bystander with an erasing
  weapon (a named bystander object, sent by a test trigger) starts erasing *G*. (A unit of the
  human house is not used: its destruction later would be event (g) or non-event (o) of the
  observed house.) Expected: the icon of *Y* appears in the first house update after the
  erasing starts. Then *V* enters a test transport (claim 12 (m): *Y* stays offered, although the
  refinery group is no longer met), and the erasing unit is destroyed by a test trigger before *G*
  is erased, so the erasing stops. Expected: the icon of *Y* is gone in the first house update
  after the erasing stops.
- Run 13 (k, a superweapon suspended): the house owns a test building with a test superweapon that
  depends on power (S-0405 claim 15) and is charging, and one power plant whose output just meets
  the house's drain. A test trigger damages the power plant, so that its output falls below the
  drain (S-0405, damaged power plants) while no building leaves the map or is switched. Expected:
  the icon of *Y* appears in the house update in which the house's low-power state begins (its tick
  is S-0405's; the superweapon's charge is held from then). Second game, the same with the
  superweapon building replaced by a test building of the same drain and no superweapon:
  the icon of *Y* does not appear until the control event (the onset of low power alone is no
  event).
- Run 14 (o, seen as a disappearance): as run 8, but instead of entering a transport the *V* (its
  type has no build limit, so it is never at limit) is destroyed by a test trigger. Expected: the
  icon of *Y* stays until a later event of claim 12 (a test building put on the map), and is gone
  in the first house update after it. With `quirks.unit_changes_no_recheck` off it is gone in the
  first house update after the destruction.
- Falsified if: in runs 1 to 5 the icon appears later than the first house update after the event
  (it would then wait for some other event) or never, or in run 2 at the start of the sale; in run 6
  it appears before the later event; in run 7 (default switches, no independent interruption)
  the icon is gone or the item refunded before the removal of *B*, the item is not refunded in
  the removal tick, the icon is still shown after the first house update after removal, or a
  second refund occurs at the later cancellation boundary; in
  runs 8 and 14 the icon is gone before the later event, or still shown after the first house
  update that follows it; in runs 9 (first and third games), 11, 12 and 13 the icon of *Y* waits
  for the control event after the candidate event (the event is missing from the original's
  list), or, in run 13, appears in a later house update than the one in which low power begins,
  or, in the second game of runs 9, 11 and 13 and in run 10, it changes before the control event
  (the original makes a recheck that the table omits); each switch-off variant gives a different tick than the one
  stated. The original's behaviour in runs 6 to 14 has not been observed yet, so the confidence of
  the claims they test is medium (low for run 13) until measured.

## Open questions

1. The original's own list of events that call for a recheck (claim 12) may hold more events than
   the table, for example other trigger actions. Because a recheck works out the whole taken-up set
   (claim 12), every recheck the original makes and the table omits is an observable difference
   from the original, as is every event of the table that the original lacks: while the set is
   stale, either one moves the tick at which an icon appears or disappears. Candidates are found
   by probing: from the stale starting state of T14 (runs 9 to 13), cause the candidate event alone
   and see whether the icon of *Y* changes before the control event. An event found that way is
   added to the table in a revision. For our engine the table is the whole list.
2. Whether a building in its construction state can serve as a factory for claim 1 (c) (it is on
   the map and switched on) was not measured.
3. Which building counts as the most recently acquired for claim 8 (c) after a capture, a deploy
   or another transfer of ownership was not verified; T8 (e) uses two buildings placed one after
   the other. A measurement would capture an older building after a newer one is placed and
   repeat (e).
4. S-0004 claim 11 suppresses a newly available icon when its tab is full. The admission rules
   here cover types with space in their tab; they do not establish offer membership, order legality,
   cancellation or later refill for a suppressed type. S-0004 scenario F4 tests suppression.
   S-0411 claim 2 leaves later admission unresolved. Those cases need further investigation
   before a complete capacity-aware taken-up-set rule can be stated.
