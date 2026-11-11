---
id: Q-2999
title: Is there a ceiling on the particles of a spark burst, a railgun line and a smoke next-type loop?
status: open
related: [S-0616]
---

## Need

Three of S-0616's counts come straight from rules data, and the spec gives no ceiling for any of
them (GitHub issue #198):

- F10: a spark burst makes *c* = (|*r*₀| rem *h*) + *h* sparks, *h* = trunc(κ / 2). A particle cap
  near 2³¹ asks for up to 2³¹ − 1 sparks in one turn. Claim 20 already overrides the *h* = 0 case
  for that reason ("a count of up to 2³¹ sparks, which no engine can make").
- F12 and claim 17: a railgun line makes *n* = trunc(*L* × *p*) particles. A large
  particles-per-lepton value over a long shot asks for millions in one turn.
- Claim 8: a dead particle in a smoke system whose type names a next type splits into two of that
  type. A type that names itself (or a loop of types) doubles the system's particles once per
  lifetime for ever, and the system never leaves play.

## Questions

1. Does the game limit the sparks of one burst, the particles of one railgun line, or the
   particles of one system? If so, what is the limit, and which draws does a limited burst or line
   still take?
2. A smoke particle type whose next type is itself: does the system grow without end, and does it
   ever leave play?
3. Does the game refuse or change such data when it loads the rules?

## Possible observation

Test rules layers with an invented spark system of particle cap 100 000, an invented railgun system
of 10 particles per lepton, and an invented smoke particle type whose `NextParticle` names itself.
Show each once and record the particles drawn per tick (or whether the game stops responding).

## Current workaround

`// nocite: our own input validation (#198)`, in `particles.rs`:

- A spark burst or a railgun line makes at most 4096 particles (`particles::BURST_CEILING`). A
  burst whose *c* is larger still takes its count raw value and the three discarded raw values,
  then makes 4096 sparks, each with its own draws, in the form of claim 20's *h* = 0 override; a
  line whose *n* is larger makes 4096 particles, each with its seven ranged draws and its creation
  draws. Counts up to 4096 are unchanged. 4096 is far above any count we know shipped data to
  give.
- At rules build our engine marks every next-type link that lies on a loop of particle types. A
  successor made through a marked link carries a saved mark (`gas::Particle::looped`). A marked
  dead particle whose next type is reached through a marked link is removed without splitting
  and takes no draw for it. So each emitted particle leads to at most two of the loop. Links on no
  loop are unchanged. The importer warns about each loop.

Tests in `crates/deterministic/sim/tests/particle_bounds.rs`.
