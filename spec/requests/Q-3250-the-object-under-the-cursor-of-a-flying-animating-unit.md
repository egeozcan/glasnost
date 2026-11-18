---
id: Q-3250
title: How the game finds the object under the cursor for a flying, animating unit
status: open
related: [S-0214, S-0102, S-0101]
answered-by: []
---

## Need

S-0214 Terms defines the *clicked object* only as "the object that the game finds under the
cursor at the moment of a click, from the cursor's position on the battlefield". The same object
decides what the cursor shows while the pointer hovers (S-0214 claim 1). No approved spec says
how the game finds that object.

Our engine finds it from the drawn pixel. `Session::pick` (`crates/app/src/game/session.rs`)
asks `Scene::object_at` which object's drawing passed every drawing test at the pointer's
screen pixel in the frame being shown (S-0102 claims 9, 10 and 31; design: render (#68)).
Shadows, health bars and other status drawings, and effects such as projectiles and
animations, never supply the object.

Issue #435 reports that with the pointer held still over a rocketeer that hovers and attacks,
the cursor keeps switching between the select cursor and the plain arrow. A trace on a
synthetic map (`crates/app/src/mapview/hover_pick_tests.rs`) finds the following:

- The pick is stable while the unit's drawn pixel at the pointer stays the same. Over 324
  frames of a hovering, firing jumpjet unit whose body is opaque throughout, every frame picks
  the unit. That holds although its shots' effects pass over the pointer in 9 of those frames.
- The pick reads the scene of the frame being shown. A frame put off by a tick is drawn before
  any pick, and the drawing does not interpolate between ticks.
- A hovering jumpjet unit's height keeps changing (in the trace, between 460 and 540 leptons,
  which moves its body by up to 12 screen rows). Its drawn frame also changes as it fires. So a
  still pointer near a real sprite's edge, or over a transparent gap in it, lies on the unit's
  pixels in some frames and off them in others.
- A tree in front of the point below the unit hides it by depth, as S-0102 claims 10 and 11
  say. Over a tall tree, the trace picks the tree in every frame. Near the tree's edge, the
  pixel goes to the tree or to the unit depending on the unit's height.

Each such change flips the hover between "object" and "cell", and the cursor follows. If the
original finds the object some other way, our cursor flickers where its cursor does not.

## Questions

1. Does the game find the object under the cursor from the pixels of the frame it shows, so
   that a transparent pixel of the object's sprite, or a pixel where terrain, a tree or a
   building hides it, is not over the object? Or does it test a region per object that does not
   follow the shown pixels (for example a box around the object's drawn frame, or around its
   position)?
2. If it uses a region, what is it for an infantry unit, a vehicle and an aircraft in flight,
   and does it move with the object's height lift (S-0101 F1)?
3. Is the object under the cursor found again on every drawn frame while the pointer stays
   still, or only when the pointer moves (or at some other interval)?
4. Where a tree or building hides part of a flying unit, is the hidden part still over the
   unit?

## Possible observation

On a flat test map with no fog, place an own rocketeer (or another jumpjet infantry type) in
flight over open ground, with an enemy target in range so that it hovers and fires. Select a
different own ground unit, so that the pointer over the rocketeer shows the select cursor and
over open ground the move cursor. Put the pointer:

- (a) on the middle of the rocketeer's body;
- (b) one pixel inside its outline at the top edge;
- (c) on a transparent pixel between its arm and its body (a pixel that shows the ground in a
  screenshot);
- (d) one pixel outside its outline.

Hold the pointer still for at least 10 seconds in each place and record the cursor image every
frame, together with the rocketeer's frame and height if a script can read them. Repeat (a) and
(b) with the rocketeer over a tall tree that hides its lower body.

- A cursor that flips with the rocketeer's frames or height in (b) and (c) supports question 1's
  pixel answer.
- A cursor that stays the select cursor in (b), (c) and (d) supports a region, and the
  distance at which (d) stops showing it gives the region's size.

## Current workaround

None. The pick stays pixel-exact, as S-0102 claims 10 and 31 and design render (#68) describe.
`the_pick_over_a_hovering_firing_jumpjet_stays_the_unit` guards that the shots' effects drawn
over the unit do not move the pick. So any flicker that remains comes from the pixels shown at
the pointer: the unit's own, or those of what hides it.
