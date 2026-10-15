---
id: Q-0711
title: The map-generator stream's seed in a skirmish on an ordinary map
status: open
related: [S-0202, S-0511]
answered-by: []
---

## Need

S-0511 claim 20 takes low-bridge repair's piece choices from the map-generator stream (S-0202
claim 13). On a map that is not a random map the seed setting is 0, so the stream is seeded from
a ranged draw R(0, 65535) of the unsynchronised random stream "taken after that stream has been
seeded for the game". Our simulation does not run the unsynchronised stream (it serves
presentation and setup only, S-0202 claim 11), so it cannot know how many values that stream has
taken before the draw. The code in `crates/deterministic/sim/src/world.rs` (the map-generator
stream) needs a seed that every machine agrees on.

## Questions

1. At which point of game start is the unsynchronised stream's R(0, 65535) taken: before any other
   use of that stream (so as its first ranged draw after seeding), or after the map load's
   presentation draws?
2. Is the map-generator stream seeded once per game and never again during play?

## Possible observation

S-0202 scenario R9 e: read the map-generator seed at the first tick of several games started with
known game seeds, and compare it with the first and later ranged draws R(0, 65535) of a stream
seeded with the same seed.

## Current workaround

The stream is seeded once at game start from the first ranged draw R(0, 65535) of a fresh stream
seeded with the game seed, as if the unsynchronised stream had taken no value before it. The choice
is marked in the code with this request's ID.
