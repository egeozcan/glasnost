---
id: M-0026
title: Gas clouds made on and under a high bridge's deck, and a chaos drone deployed on the deck (S-0613 G14)
topic: combat
specs: [S-0613, S-0610, S-0306]
requests: [Q-3206]
verdict: confirms
value-visibility: committed
measured-on: 2026-11-17
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-17
---

# M-0026: Gas clouds made on and under a high bridge's deck, and a chaos drone deployed on the deck

This record kills an infantry unit with a virus death on a high bridge's deck, and in a second
setup on the ground under the same deck, and follows every gas particle of its clouds tick by tick.
In a third setup a chaos drone deploys on the deck. On the deck, every particle starts at the dead
unit's position, at the deck's height, with a vertical speed of zero; it is below the deck's height
after its first odd-tick move and sinks to 5 leptons above the ground under the bridge, where it
stays for the rest of the run, also where it drifts beyond the bridge's cells. Its recorded height
equals S-0613 F4 and F5, with the ground under the bridge as the terrain, at every tick of every
particle. Neither cloud ever hit the infantry standing on the deck in the cloud's own cell,
although hits fell due while particles were in that cell. Under
the bridge, every particle rises to 5 leptons above the ground and stays there. The chaos drone
made no particle. On each of its pulses an animation appeared next to it at the deck's height and
kept that height and its position for its whole life, and the infantry on the deck in the next
cell went berserk at the first pulse and stayed berserk to the end of the run. This is what S-0613
claim 19 gives with `quirks.gas_through_bridge_deck` on, and what S-0613 claim 20 gives for an area
berserk weapon.

## Setup

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness
  (PROVENANCE §7).
- Data: stock rules and art, no mods, no loose rules or art files. Every gameplay and AI change of
  the harness is off, apart from the peace setting under *What is not stock*.
- Map: the stock multiplayer map `Kaliforn.mmx` (sha256
  `3da7ebe513ae2e6a78ac6b8343ff3d3c6d3e33bb7f518fe4e37d3e87a1e64497`), with its infantry list
  replaced by the test infantry below; its other objects are kept. On this map an east-west road
  crosses a lower north-south road on a high bridge. The *test cell* is a cell in the middle of the
  bridge that carries the deck over the lower road. Every cell of the bridge, and the cells next to
  it that the clouds reached, are flat and at the same height level, so the ground under the deck
  and beside the bridge is at one height. The deck is four height levels above it (S-0616 claim 14).
  The cells, the levels and the bridge's extent are not given here.
- Game: a skirmish at the fastest game speed. Every player starts with 0 credits, at tech level
  10, with superweapons, crates and the short game off, bases on and the map revealed. The setting
  that keeps the computer players at peace with the human player is on (*What is not stock*).
- Houses: the human player (British), whose virus sniper and chaos drone are the only objects that
  fire, and who owns one power plant far from the bridge so that it is not defeated; two computer
  houses (Americans, Russians), each owning only a power plant and a construction yard far from the
  bridge; and the map's neutral house, which owns the test infantry. The test infantry are GIs (E1)
  and conscripts (E2), map-placed with full health, in the mission that keeps a unit where it is
  (*sticky*); a unit on the deck is placed as on the bridge, a unit under it as on the ground.
- Setups:
  - *deck*: in the test cell, GI A and conscript B on the deck, GI C and conscript D on the ground
    under it. At tick 30 the human's virus sniper (VIRUS), standing on the lower road beside the
    bridge, is ordered to attack A.
  - *under*: in the test cell, GI A and conscript C on the ground under the deck, conscript B on
    the deck. At tick 30 the virus sniper is ordered to attack A.
  - *drone*: the human's chaos drone (CAOS) is created on the upper road west of the bridge and
    ordered at tick 1 to the test cell, which it reaches on the deck; once it stands there it is
    ordered to deploy (the order tick is not given here). GI B stands on the deck in the cell east of the test cell, conscript C on the ground
    under the test cell. Nothing else fires.
- Runs: deck, under and drone with three seeds each (9 runs) to a tick limit of 900, and one
  shorter run of the deck setup with a fourth seed (the *short deck run*), to a tick limit of 200.
  The game plays on for some ticks after the limit, and the recording covers them: to tick 911 in
  the nine runs and to tick 303 in the short deck run. The short deck run is not counted in any
  *n* below.
- What is not stock:
  - no menus: the skirmish setup comes from a settings file
  - the map's infantry list is replaced, and the human's and the computer houses' objects above are
    created on their cells; the computer houses' starting construction vehicles are gone from tick 1 on
  - the attack, move and deploy orders above, given by the harness at the stated ticks
  - the computer players are kept at peace with the human player by a harness setting
  - the run ends at a fixed tick limit
  - a per-tick recorder reads state only
- Tick convention: a row stamped tick *f* is read after tick *f*'s logic has run.

## Procedure

The recorder logs, every tick: every particle in play, of any kind and in any system, with its
type, its system, its position and height in leptons, its velocity, the cell it is in and its
damage countdown (S-0613 claim 13); every animation's type, position and height; every unit's position, height,
cell, health and berserk state; every hit with its warhead, its attacker (or none) and the health
before and after; and, once, each cell's height level, slope and whether it is a cell of a high
bridge.

1. For each particle, find its first row (its *creation tick*) and compare its position and height
   with the dead unit's last row.
2. For each particle, run S-0613 F4 and F5 from its first row, taking *H* as the recorded height
   level of the cell it is in times 104 leptons, and compare the result with every later row's
   height and vertical speed.
3. List every hit after the death, with the unit hit, the cell it stood in and its height.
   For each particle, find the ticks at which its damage countdown restarts, which are the ticks
   at which a hit of it falls due (S-0613 claim 13), and list the units standing in its cell then.
4. In the drone runs, list every particle, every animation that is not a unit's own, each
   animation's height and position over its life, every hit with the drone as attacker, and the
   ticks at which each unit is berserk.

## Result

**Deck setup** (deck runs; the short deck run gives the same rows up to its end). Heights are
given against the deck's height (*deck*) or the ground under the bridge (*ground*); the deck is
416 leptons above the ground. Every particle of the three runs is counted.

| Quantity | Unit | n | Mean / value | Spread | Tolerance for tests |
|---|---|---|---|---|---|
| Particle's position at its creation tick against the dead unit's last position | leptons | all, 3 runs | equal (height at the deck) | none | exact |
| Vertical speed at the creation tick | leptons per tick | all, 3 runs | 0 | none | exact |
| Parity of the particle's first update (the tick after the creation tick) | | all, 3 runs | even | none | exact |
| Height after the first odd-tick move, the first update being on an even tick | leptons from deck | all, 3 runs | −3 | none | exact |
| Sinking once the vertical speed is −5 | leptons per two ticks | all, 3 runs | 7 | none | exact |
| Ticks from creation until the height is 5 leptons above the ground, the first update being on an even tick | ticks | all, 3 runs | 122 | none | exact |
| Height from then to the end of the run | leptons from ground | all, 3 runs | 5 | none | exact |
| Rows whose height or vertical speed differs from S-0613 F4 and F5 | rows | all, 3 runs | 0 | none | exact |
| Runs with a hit falling due while the particle and B were in the same cell | runs | 3 | 3 | none | exact |
| Hits by the gas on B (on the deck, in the cloud's cell, there for the whole run) | hits | 3 runs | 0 | none | exact |

In every deck run the first hits fell due in the test cell with B there and every particle below
the deck, and later ones fell due there as well while particles stayed in that cell. Some particles drifted across the bridge's cells and, in two of the runs, beyond them; they kept a
height 5 leptons above the ground there too. The ground beyond the bridge is at the same level as
under it, so these runs do not show a change of height at the bridge's edge.

**Under setup** (under runs).

| Quantity | Unit | n | Mean / value | Spread | Tolerance for tests |
|---|---|---|---|---|---|
| Particle's position at its creation tick against the dead unit's last position | leptons | all, 3 runs | equal (height at the ground) | none | exact |
| Height after the first odd-tick move, and to the end of the run | leptons from ground | all, 3 runs | 5 | none | exact |
| Rows whose height or vertical speed differs from S-0613 F4 and F5 | rows | all, 3 runs | 0 | none | exact |
| Runs with a hit falling due while the particle and B were in the same cell | runs | 3 | 3 | none | exact |
| Hits by the gas on B (on the deck, in the cloud's cell, there for the whole run) | hits | 3 runs | 0 | none | exact |

**Drone setup** (drone runs). The drone stood on the deck in the test cell from before its first
pulse to the end of each run.

| Quantity | Unit | n | Mean / value | Spread | Tolerance for tests |
|---|---|---|---|---|---|
| Particles of any kind | particles | 3 runs | 0 | none | exact |
| Height of each pulse's animation, at every tick of its life | leptons from deck | every animation, 3 runs | 0 | none | exact |
| Animations whose position changed during their life | animations | 3 runs | 0 | none | exact |
| Pulses whose animation appeared outside the drone's cell | pulses | 3 runs | 0 | none | exact |
| Pulses without a hit on B by the drone in the animation's first tick | pulses | 3 runs | 0 | none | exact |
| B berserk from the first pulse's tick to the end of the run | runs | 3 | 3 | none | exact |

Each of these hits names the drone as attacker and a berserk-causing warhead (S-0610 claim 8). The
drone's pulse cadence, the animation's life and the health values are not given here (*Notes*).

**What these runs do not show.**

- In every run of every setup, the infantry placed on the ground under the deck (C and D) moved out
  of the test cell early in the run, before the first gas hit of any run, and in the drone runs as
  well, where nothing died. So no run records a gas hit on an object under
  the bridge in the cloud's cell, or what the drone's pulses do to an object under the bridge.
- The only gas hits recorded are in one deck run: C and D, standing on the ground off the bridge
  in a cell they had walked to, were hit by the gas, with no attacker, after a cloud at 5 leptons
  above the ground had drifted into that cell.
- B was on the deck while the clouds were level with it only on their creation tick and the tick
  after it, before any hit was due, so these runs do not show whether a cloud level with the deck
  hits an object on the deck.
- One map and one bridge; flat ground at one level on both sides of the bridge's edge; no screen
  record, so nothing here is about drawing.

## Evidence

- 10 recordings, played on 2026-11-17: 3 deck, 3 under, 3 drone and the short deck run. The
  recordings stay local, and no log hash is given.
- Three earlier drone runs are void and not counted: no drone was in play. They placed the drone
  as a map unit owned by the human's country, and a multiplayer game did not create it. The counted
  drone runs create it after the scenario has loaded instead, as above.
- No counted run crashed or stopped before its end. In every deck and under run the virus sniper
  killed A, A died a virus death, and no other unit died.

## Notes

- **Values left out.** The tick of A's death, the ticks from the death to the clouds' creation and
  the number of particles per death come from the stock weapon, warhead and animation data, so this
  record gives none of them, and gives counts as "all" rather than as numbers. The drone's pulse
  cadence and the number of pulses in a run would bound its weapon's rate of fire, and an
  animation's life comes from art data, so neither is given. Health values and the gas hits'
  spacing are left out for the same reason.
- **Reproducing it:** the setup uses a stock map, so a comparison against this record runs in the
  local lane, with the map read from the game directory; a test on a map of our own measures
  something else and is not checked against it. The Result gives heights against the deck and the
  ground rather than this map's height levels; these runs cover one map and one bridge.
- **Why the ground infantry left** is not established here; S-0613 G14's falsifier on C assumes
  that C stays in the cell.
