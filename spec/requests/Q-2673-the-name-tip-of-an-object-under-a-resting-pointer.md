---
id: Q-2673
title: The name tip shown when the pointer rests on a unit or building
status: open
related: [S-0105, S-0411, S-0111, S-0516]
answered-by: []
---

## Need

The lead's play note 54 of 2026-11-05 (M8 A4 round 8): "units do not get a little name tag when
hovered long". No spec gives hover tips over the battlefield. S-0105's Open questions say so ("Hover
tip name (pointer only). This spec does not specify hover tips. The interface spec that takes them
should check whether a multi-turret type's hover-tip name follows its selected turret index (claim
43).") and S-0411 covers only the sidebar's. Our view showed nothing for an object under a resting
pointer.

## Questions

1. How long must the pointer rest on an object before its tip shows, in real time or in game
   ticks, and does the wait start again when the pointer moves within the same object, or only
   when it moves to another object or off it? Does a game speed change or a pause change it?
2. What text does the tip show: the type's display name only, or more (owner, health, veterancy)?
   For a multi-turret type, does the name follow its selected turret index (S-0105 claim 43)?
3. Which objects get one: own, allied and enemy units and buildings, civilian and neutral
   buildings, terrain objects (trees), overlays, an occupied building (its name or its
   occupants')? Does a cloaked object drawn for its owner, or a disguised unit seen by an enemy
   (S-0516 claim 35), show its own name, the disguise's, or none?
4. Where is it drawn relative to the pointer (offset, which side), how is it kept on the screen
   near the edges, and what are its look (font, colours, frame) and layer against the sidebar and
   other interface drawings?
5. When does it go: the pointer leaving the object, any press, the object dying or leaving the
   map, a selection change? Is it shown while a selection box is dragged, a placement or a
   superweapon aim is active, or the pointer shows an order cursor?

## Possible observation

In a skirmish, rest the pointer on an own tank, an enemy tank, an own building, a tree, a
disguised enemy spy and an own cloaked unit, each for 3 seconds; record when the tip appears
(from a screen capture at 60 frames a second), its text and its screen position against the
pointer, and repeat near each screen edge and at the slowest and fastest game speeds.

## Current workaround

Our design (M8 A4 round 8 note 54), cited `design: M8 A4 round 8 note 54`: once the pointer has
rested 700 ms of interface time on the same unit or building (the object the hover health bar of
S-0111 claim 3 marks), a one-line tag with its type's name shows below and right of the pointer,
in the sidebar tooltip's look, kept inside the battlefield. None for terrain objects, for an
object that appears disguised to the local player's house (S-0516 claim 35), while the options
menu is open, the keyboard aims, a box is dragged or the right button is held. The name is the
one the sidebar shows (the type's name, until Q-2674 gives display names).
