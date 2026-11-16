---
id: Q-3158
title: What a transport's passenger shows when its own house's defeat hits it inside
status: open
related: [S-0414, S-0312, S-0507, S-0107, S-0605]
answered-by: [S-0312]
---

## Need
S-0414 claim 5 offers the defeat hit to every object of the house, passengers off the map included,
and claim 15 leaves a passenger's death with its carrier to S-0312. S-0312 claims 5 and 12 say what a
passenger shows when it dies with its transport: it is credited, its destruction raises are made, it
shows no death effects of its own and takes no draws for them, and it is removed. No claim says what a
passenger shows when the defeat hit reaches it on its own while it rides, before or without its
transport's hit (the transport may belong to another house, may be exempt from the sweep, or may come
later in the sweep's order).

The garrison occupant has an answer: S-0508 claim 19 kills it inside by an individual killing hit with
no eviction. An absorbed passenger has one too: S-0605 claim 15 and scenario A11 delete it and drop the
count at once. For a transport's passenger the ordinary path would run S-0312's death effects (debris,
death weapon, animations) at the position where the passenger went inside, and an infantry passenger
whose death asks for a die sequence (S-0107 claim 17: a type marked not human, or an infantry-death
kind of 1, 2 or 9) would stay in play while the sequence plays (S-0203 claim 6 (c)). A passenger of a
closed transport takes no turn, so the sequence never completes and the dead unit keeps its seat.
(Issue #383.)

## Questions
1. When a defeat or surrender hit kills a passenger of a closed transport on its own, does it show
   death effects (debris, death weapon, death animations, an infantry death sequence), or is it removed
   as a passenger that dies with its transport is (S-0312 claim 5)?
2. The same for a passenger of an open-topped transport, which is in play and takes turns
   (S-0507 claim 14, S-0203 claim 3).
3. If either shows a death sequence, where does it play, and when does the passenger leave the
   transport's list and free its seat?

## Possible observation
S-0414 scenario V11 part b with a passenger type marked not human, in a closed transport and,
separately, an open-topped one: record the passenger's death animation and sequence, any debris, and
the transport's passenger count in the tick of the defeat and the ticks after it.

## Current workaround
A passenger that a killing hit reaches while it rides any transport, or while it is absorbed in a
building, gets the garrison occupant's treatment: health 0, destruction raises as the hit allows, no
death effects of its own and no draws for them, and it is removed at once, which frees its seat
(`death::kill_in`, `death::passenger_removed`, in `crates/deterministic/sim/src/combat/death.rs`).
The absorbed passenger's treatment is settled by S-0605 claim 15; the transport passenger's is the
interim.
