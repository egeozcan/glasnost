---
id: Q-0724
title: What a cloak radius of zero or below does
status: open
related: [S-0516]
answered-by: []
---

## Need

The S-0516 Open questions ask what a negative cloak radius does. The spec also leaves two
more things open:

- whether a radius of 0 covers the generator's own cell;
- which radii the field passes through as it grows.

The code needs the rule in `crates/deterministic/sim/src/cloak/field.rs` and in `cloak/mod.rs`
(`field_circle`).

## Questions

1. Take a generator type with a cloak radius of 0. Does it cover any cell, such as its own centre
   cell?
2. With a negative radius, is there no field, a field of some other size, or something else?
3. As the field grows from radius 0, does it pass through every radius 1, 2, …, r? Is the first
   ring radius 1?

## Possible observation

Use a test file that gives a generator type a cloak radius of 0, then −1. Stand a unit of the
generator's owner that cannot cloak by itself on the generator's neighbouring cells and on its
foundation.

## Current workaround

A radius of 0 or below gives an empty field. Growth adds the circle of radius 1, then radius 2, and
so on up to the type's radius. The choice is marked in the code with this request's ID.
