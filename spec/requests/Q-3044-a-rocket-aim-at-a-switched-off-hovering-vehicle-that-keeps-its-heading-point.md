---
id: Q-3044
title: Where does a rocket aim at a switched-off hovering vehicle that keeps its heading point?
status: answered
related: [S-0506, S-0503, S-0306, M-0016, M-0003]
answered-by: [S-0506]
---

## Need

S-0506 claim 14 fixes a rocket's aim point at the destination's heading point: "for a unit: the
point its locomotor is moving it to at that moment (the end of the step it is on), or its position
when it is not moving". For a hovering vehicle the end of its step is its heading point *P*
(S-0503 Terms). S-0503 has two terms that could be "moving" here: *moving* (it has a destination
or a heading point, Terms) and *moving now* (moving, and its throttle *T* is not 0).

They differ for a switched-off hovering vehicle. By S-0503 claim 10 the switch-off's stop request
clears the destination and stored path but not the heading point, and claim 10's last paragraph
says such a vehicle "is moving" while it keeps either. By F1 its throttle is 0 when it is not
powered, so once it has settled its hover speed falls to 0 and it never travels to *P* or arrives
there (claim 7). It keeps *P* for as long as it stays switched off. *Moving now* does not separate
the cases either: with its step length 0 and its movement heading exactly at *P*, the stalled-start
reset (Edge cases, `quirks.hover_stalled_start_reset`) sets *T* = 1 on every tick, so such a
vehicle is moving now at the end of each of its updates.

GitHub issue #191 asked that a rocket aimed at a moving hovering vehicle use *P*, not the
vehicle's position. Built with the broad reading (*P* whenever the vehicle has one), the oracle
arena's stock row for ROBO against V3 in M-0003 (restated by M-0016) stops reproducing. There the
column side wins 12 of 12 in stock, and ours did so before. The robot tanks there are switched off
(they fire no shot). They chose a heading point in their first ticks, while still above the
ground and so powered, then settled. With the broad reading every rocket aims at a heading point
the vehicle never reaches, and all 12 games run to the tick cap.

Claim 10's last paragraph settles whether shots at such a vehicle are led (S-0306 claim 3 b). It
does not say whether a rocket's aim point is the heading point.

## Questions

1. For S-0506 claim 14, is a hovering vehicle "moving" when it has a destination or a heading
   point (S-0503 *moving*), or only when it is moving now (*T* ≠ 0)? Or is it some other test, for
   example whether it is powered?
2. A switched-off hovering vehicle, settled at hover height 0, that keeps a heading point: is a
   rocket's aim point its heading point or its position?
3. A powered hovering vehicle with a heading point but throttle 0 for a tick (its movement
   heading more than 45° from the direction to *P*, F1): is the aim point *P* or its position?

## Possible observation

M-0016's ROBO against V3 rows already show the outcome for question 2: in stock the launchers win
every game. A recorder run of one launcher against one switched-off hovering vehicle that holds
a heading point (record *P*, *T*, the position and the rocket's detonation point) answers it
directly. Question 3 needs a powered hovering vehicle turning a sharp corner when the launch
happens.

## Current workaround

A hovering vehicle's heading point is the aim point only while it is powered (S-0503 Terms: not
switched off, or above hover height 0). Otherwise the aim point is its position. This keeps
#191's case (a powered hovering vehicle travelling to *P*) and M-0016's stock row. A switched-off
vehicle still settling is aimed at *P*; question 2 covers only the settled one. The code (`rocket::heading_point_with`) and the
tests `rocket_aim_at_a_moving_hover_is_its_heading_point` and
`rocket_aim_at_a_switched_off_hover_is_its_position` cite this request.
