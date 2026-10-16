---
id: Q-0978
title: Whether a jumpjet in the air is listed in the cell below it
status: open
related: [S-0502, S-0209]
answered-by: []
---

## Need

S-0209 claim 13 says an ordinary aircraft in flight is in neither cell-layer list (its listing
height is positive). It says nothing about a jumpjet. S-0502 F3's allowance of 85 leptons for
"infantry, a vehicle or an aircraft listed in the cell's ground layer" is read for the cell below
and ahead of a flying jumpjet, so the answer decides whether a hovering jumpjet counts for its
own cell's clearance (85 leptons more height) and for another jumpjet's look-ahead, and whether
ground units meet it in a cell's list.

## Questions

1. Is a jumpjet in the air listed in the ground layer of the cell under its position?
2. If it is, does its own listing count in its own cell's clearance (so every hovering jumpjet
   hovers 85 leptons above its cruise height)?

## Possible observation

A hovering jumpjet over open flat ground: its height above the surface against its cruise height;
a second jumpjet flying over it.

## Current workaround

A jumpjet is listed (and marked) only while it is on the ground; in the air it is in no list
(`jump::leave_ground`, `jump::settle`), so it never counts for a clearance.
