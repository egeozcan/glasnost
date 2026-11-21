---
id: Q-3349
title: The cursor under a held right press before its scroll drag, and the hover under a left press the route mode takes
status: answered
related: [S-0214, S-0213, S-0101, S-0111]
answered-by: [S-0214]
---

## Need

S-0214 r96 claim 35 (e) says that while a battlefield press (Terms) of either button is held,
the hover is not looked up again, and that while a left battlefield press is held the cursor
shows the image that the press's own resolution at its pixel (claim 26) gave it. It leaves three
cases open, which our client must draw (`Session::update_cursor` and `Session::frame_hover` in
`crates/app/src/game/session.rs`):

- "While a right battlefield press is held, the cursor is S-0101 claim 17's once its scroll drag
  is active, and before that is not stated here." T27 part (g)'s right-press run does not compare
  the cursor.
- "A left press that the route mode takes at the press (S-0213 claim 16b) is not a battlefield
  press, and this claim does not say whether the hover is looked up while it is held" (Open
  questions; Edge cases).
- "When claim 18's band box starts, outside route-editing mode, the cursor becomes the ordinary
  arrow." What the cursor shows once a band box starts while the route-editing key is held (an
  ordinary press in route mode, S-0213 claim 16b) is not stated.

## Questions

1. While a right press that began on the battlefield view is held and its scroll drag has not
   started (the pointer has stayed within S-0214 F8's distance of the press pixel), which cursor
   image shows: the image the last hover lookup before the press gave, an image chosen at the
   press pixel as for a left press, the ordinary arrow, or another? Does the image change when
   the object under the pointer leaves play or cloaks, or when the pointer moves within that
   distance onto another object or cell?
2. While a left press that the route mode takes at the press is held (it takes a route point, a
   join or a loop close, or it is refused, S-0213 claim 16b), is the object under the cursor
   looked up for every drawn frame, as with no press? Does the cursor image follow the pointer,
   and does a hover health bar show on an object the pointer moves onto?
3. With the route-editing key held, once an ordinary left press on the battlefield becomes a band
   box (claim 18), does the cursor become the ordinary arrow until the release, as outside
   route-editing mode, or does it keep the press's image?

## Possible observation

As S-0214 scenario T27 part (g), with the cursor image recorded per tick:

1. The pointer rests 30 ticks over V's point, then the right button is pressed there and held
   for 60 ticks without moving (no scroll drag starts), while V drives out of the pointer's region
   and cell; then released at the same pixel. A second run moves the pointer 3 pixels, onto an
   empty open cell, while the right button is held.
2. With a unit selected and the route-editing key held, press the left button on an existing route
   point so that the press takes it, hold it, and drive a unit of the player's under the pointer;
   record the cursor image and the hover health bar per tick until the release.
3. With the route-editing key held and nothing selected, press the left button on an empty open
   cell, drag 30 pixels so that a band box starts, and record the cursor image until the release.

## Current workaround

1. A right battlefield press keeps, until its scroll drag starts, the image its own resolution at
   the press pixel gives, as a left battlefield press does (claim 35 (e)); once the scroll drag is
   active the cursor is S-0101 claim 17's. Marked `design: render (Q-3349)`.
2. A left press that the route mode takes at the press does not stop the lookup: the hover is
   looked up for every drawn frame and the cursor follows it, as with no press. Marked
   `design: render (Q-3349)`.
3. In route-editing mode a band box does not change the cursor: it keeps the press's image, by
   claim 35 (e)'s general rule for a held left battlefield press. Marked `design: render (Q-3349)`.
