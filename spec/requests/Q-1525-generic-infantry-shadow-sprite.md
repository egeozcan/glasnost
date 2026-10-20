---
id: Q-1525
title: Which sprite file is the generic infantry shadow, and how is it drawn
status: answered
related: [S-0107, S-0102, S-0103, S-0007]
answered-by: [S-0107]
---

## Need
S-0107 claim 19 says an infantry unit whose height above the surface is above 0 draws no shadow
frame of its own body sprite and draws "a generic infantry-shadow sprite ... image frame 1, at the
ground position below the unit" instead. S-0102 claims 13 and 20 give its depth bias and its
placement *L*(*h*) rows below. No spec names that sprite's file (S-0007 lists no such name, and no
import mapping gives a key for it), its palette or how it darkens what is under it, so the client
(`crates/app/src/mapview/build.rs`, `infantry_items`) cannot draw it. The lead's review of
2026-10-19 found the paratroopers drawn with no shadow while they drop.

## Questions
1. Which image file is the generic infantry shadow drawn from? Is the name fixed, or given by a
   rules or art field (which section and key, and what is used when it is absent)?
2. Is it drawn as a darkening like an own shadow (S-0102 claims 20 and 28), or with a palette (which
   one, at which brightness)?
3. Is it drawn while the unit's height above the surface is below 0 (S-0107 claim 19 covers only
   heights above 0)?
4. Does its size or frame change with the unit's height, or is image frame 1 always used?

## Possible observation
Drop paratroopers over flat clear ground at the highest detail level; capture the ground under one
trooper every few frames of its fall and compare the shadow pixels with the frames of candidate
image files, darkening and under the anim and unit palettes.

## Current workaround
Interim rule (docs/development.md, *Visual review 2026-10-19*): an infantry unit whose height
above the surface is not 0 draws no body-sprite shadow (S-0107 claim 19), and the generic shadow
is not drawn until this request names its sprite.
