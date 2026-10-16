---
id: Q-1090
title: When the teleport-capable dock fallback of S-0207 claim 27 is tested, and whether placement's idle setup assigns a destination
status: answered
related: [S-0207, S-0502, S-0417, S-0402]
answered-by: [S-0207]
---

## Need

S-0207 revision 31 claim 27 (and S-0502 revision 7 claim 27) give the destination-refresh state and the
one-application movement-point suppression two activations: a temporary interval during successful
placement's initial idle-behaviour setup, and a fallback for a teleport-capable vehicle that attempts
entry at an unloading dock. The claims list the predicates of the fallback (the type's teleport-capable
flag, no teleport aftereffect, no magnetic lift, not seized, a contact that is a building permitting dock
unloading, a destination cell with no ground-layer vehicle, a current locomotor that is not teleporting)
and say the outcome depends on whether "temporary locomotion can finish and restore its suspended
predecessor".

The engine has the pieces (the type flag, the unloading-dock flag, the temporary driving locomotion of
a factory exit with its restoration, S-0417 claim 5, S-0207 claim 21) but no step where an entry
attempt at a dock evaluates these predicates, and the claims do not say at which moment of the dock
handshake the evaluation happens. Without that the code cannot place the check, and no ordinary play
sequence in our engine can set either pending state. `jump::take_destination`, `drive::take_destination`
and `jump::stop_ordered` therefore never see a refresh or a suppression.

## Questions

1. At which moment of a vehicle's approach to an unloading dock is the fallback tested: when the
   *enter* order is applied, when the vehicle first makes contact with the dock building, on each
   processing of *enter* while it holds the contact, or at another step? Is it tested once per entry
   attempt or repeatedly?
2. When is "temporary locomotion can finish and restore its suspended predecessor" true for a
   vehicle: only while a factory exit's temporary driving is installed and its restoration is
   permitted by S-0215 claim 13's first row, or in other states too? What does a vehicle in the
   other states do (the "otherwise" branch) when it is not in that situation at all?
3. Does the initial idle-behaviour setup of a placed vehicle (S-0502 claim 27, first paragraph) make
   a destination assignment of its own (for example an empty one)? If yes, which one and in which
   order relative to the placement draws? If no, the placement interval has no observable effect on
   any locomotor and need not be modelled.
4. For a vehicle with the drive locomotor, does the pending suppression skip the one subsequent
   nonempty point installation of S-0207 claim 25 in the same way as for a jumpjet's movement point?

## Possible observation

S-0207 D32's conditional diagnostic, with a recorder of the refresh and suppression states, if an
ordinary dock sequence establishes every predicate.

## Current workaround

Interim rule: neither the destination-refresh state nor the point suppression ever becomes pending in
our engine, and placement's idle setup is taken to make no destination assignment. An unchanged
vehicle destination assignment is always ignored. Noted in `docs/development.md` (section
*Simulation (M6c: jumpjets)*) and in `docs/m6-claims/step1e-m6c.md`.
