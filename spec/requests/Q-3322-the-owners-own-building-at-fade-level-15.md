---
id: Q-3322
title: Can the owner hover, click and pick its own building at fade level 15 that it does not sense
status: answered
related: [S-0214, S-0516]
answered-by: [S-0516]
---

## Need

Two approved specs disagree about a building at fade level 15 (S-0516 claim 18) whose owner is
the viewing house *V* and which *V* does not sense.

- S-0214 revision 87, claim 35 (d): the exclusions apply only after (a) or (c) has chosen an
  object that the viewer does not own; "None of these exclusions applies to an object that the
  viewer owns", and "The viewer can hover, click and pick its own building at fade level 15 where
  it does not sense it, since such a building stays a candidate (claim 24)". Its *Changes for
  other specs* asks S-0516 claim 26 to read "a building at fade level 15 that *V* does not own and
  does not sense", and CS9 part e to read "at level 15 without sensing the owner can pick it, and
  a non-owner cannot, allies included".
- S-0516 revision 25, claim 26 still reads "a building at fade level 15 that *V* does not sense",
  with no owner exemption, and CS9 part e still says such a building "cannot be picked even by
  the owner".

What our client does today, with the owner's own building at fade level 15 that it does not
sense under the pointer:

- The clicked object (`Session::clicked_object` in `crates/app/src/game/session.rs`) follows
  S-0214 claim 35 (d): the building is found, as for any object the viewer owns.
- The hover mark (`Session::update_marks`, same file) passes the found object through
  `glasnost_sim::cloak::view::clickable` (`crates/deterministic/sim/src/cloak/view.rs`), which
  follows S-0516 claim 26 as written. The building gets no hover health bar.
- What a click and the cursor target (`Play::target` in `crates/app/src/game/play.rs`) passes it
  through `glasnost_sim::targeting::clickable_by`, which uses the same test. The cursor's action
  and the click's order therefore fall to the cell under the pointer, as if no object were there,
  and a left click does not select the building.

So the cell's building is found and then dropped. Under S-0214 the owner would hover, select and
order onto it; under S-0516 as written it would not. A viewer that does not own it is refused
under both. The only case that differs is the owner's own building, unsensed, at fade level 15.

The change S-0214 asks for would be made in `cloak::view::clickable`, in the deterministic
crates. Our render batches do not touch those crates, so nothing has been built.

## Questions

1. Which reading holds: can the owner hover, click and pick its own building at fade level 15
   that it does not sense (S-0214 claim 35 (d) and its *Changes* entry), or not (S-0516 claim 26
   and CS9 part e as they stand)?
2. If the owner can find it: can the owner also select it (S-0214 Terms, *can be selected*), and
   does S-0516 claim 26's selection list change with it? Today that list deselects only non-owner
   viewers.
3. If the owner can find it: does an order whose target is the owner's own building at that
   level (a repair, an entry, a deploy onto it) take the building as its target, as at any lower
   fade level?

## Possible observation

S-0516 CS9 part e, as S-0214's *Changes* entry rewrites it. Take an own building that cloaks
(a cloaking generator's field, or a building type that cloaks), let it fade to level 15, and
make sure no detector of the owner senses its cells. Then hover it and record whether its health
bar appears. Left-click it and record whether it is selected. With a unit selected that can
enter, repair or deploy onto it, record the cursor's action over it.

## Current workaround

None built. The client keeps today's behaviour above: found by the clicked-object lookup, then
dropped by the hover mark and by the click's target through S-0516 claim 26's test, so the
owner's click acts on the cell. When the lead settles the question, the change goes in
`cloak::view::clickable`, as an owner exemption in its fade-level-15 test, with a test for CS9
part e.
