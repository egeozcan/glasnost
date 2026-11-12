---
id: Q-2996
title: What does an animation type do whose trailer, spawn, bounce or expiry animation leads back to itself?
status: open
related: [S-0109, S-0319, S-0203]
---

## Need

S-0319 claim 12 makes an animation's trailer in every turn on a due tick, the first turn included,
and S-0203 claim 4 lets a new animation take its first turn later in the same pass. S-0319 claim 7
makes a landing's bounce animation, claim 8 its expiry animation and claim 9 its *n* spawns, each
of which can be a bouncing type that lands again. S-0109 claim 16 switches a type to its follow-up
type. None of these says what happens when the types lead back to themselves: a type that is its
own trailer (or X names Y and Y names X), a bouncing type that spawns itself, or a trailer that
reaches the creating type again through a follow-up switch.

In our engine such data made one tick run for ever (GitHub issue #224: each new trailer's first
turn makes another in the same pass) or made the number of animations grow without end (#174:
every landing makes up to 2*c* more landings). Nothing in the shipped rules is known to form such a
loop; a mod or a mistake in a custom layer can.

## Questions

1. With `TrailerAnim` naming the type itself and a separation of 1, how many animations does one
   creation of the type make in its first tick, and in each later tick?
2. The same for two types that name each other as trailer, and for a trailer reached again
   through a follow-up (`Next`) switch.
3. A bouncing type whose spawn type (or bounce or expiry animation) is itself: does each spawn
   land and spawn again, and does that stop?
4. Does the game refuse or change such data when it loads the rules?

## Possible observation

A test rules layer gives an invented warhead animation X `TrailerAnim=X`, `TrailerSeperation=1`,
and a second layer a bouncing X with `Spawns=X`, `SpawnCount=2`. Fire a weapon whose warhead
shows X at an empty cell and record, per tick, the number of animations drawn (or whether the game
stops responding) for 30 ticks.

## Current workaround

At rules build our engine lists the links between animation types (trailer, a bouncing type's
spawn, bounce and expiry animations, and the follow-up type) and marks every link that lies on a
loop of types (a strongly connected group of the link graph). An animation made through a marked
link carries a saved mark (`anims::Anim::looped`). A marked animation takes no marked link: it
makes no trailer, spawn, bounce or expiry animation through one, and takes no draw for it (a
landing's spawn count draws are not taken). Links that lie on no loop, and every animation made
through them, are unchanged; so is the follow-up switch itself, which makes no animation. The
importer warns about each loop. `// nocite: our own input validation (#224, #174)`, in
`anims::turn` and `bounce::land`, with tests in `crates/deterministic/sim/tests/trailer_cycle.rs`
and `spawn_cycle.rs`.
