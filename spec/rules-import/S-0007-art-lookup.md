---
id: S-0007
title: Art lookup (image indirection, sprite or voxel, companion files, theater substitution)
topic: rules-import
revision: 11
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: [Q-0008, Q-0083]
supersedes: []
---

# S-0007: Art lookup (image indirection, sprite or voxel, companion files, theater substitution)

## Summary

How the original turns a type's rules and art settings into the names of the art files it loads
(sprite, or voxel model with turret, barrel and motion files), including theater-dependent renaming,
the fallbacks for missing files and the moments in a game when names are built. It covers infantry,
vehicles, aircraft, buildings, animations, overlays, terrain objects and smudges, and also
projectiles, particles, particle systems, voxel animations and the superweapon sidebar icon; it says
that weapons, warheads and tiberiums load no art file of their own (claim 32). File resolution
itself is S-0005, and drawing the loaded art belongs to the rendering specs (S-0105, S-0107, S-0108 and
S-0109; a translucent sprite is drawn as a light-glow image, S-0104 and S-0110).

## Terms

Shared terms from the glossary used here: *archive* (and *nested archive*), *art database*, *base
rules*, *engine constant*, *game*, *game-mode file*, *local player*, *loose file*, *map layer*,
*measurement setup*, *override section*, *quirk switch*, *resolve*, *rules database*, *rules layer*,
*session*, *stock data*, *theater*, *type*, *type kind*, *type name*, *type section*, *unspecified*,
*voxel model*. When a name resolves, it is matched without regard to letter case (S-0005).

- **Image name.** The name a type uses to find its art (claims 1, 9 and 10). At most 24 bytes.
  It is never empty (claim 1), except for a projectile, whose image name is empty when its section
  has no image key (claim 26).
- **Invisible projectile.** A projectile whose rules section sets the invisible flag. It is never
  drawn (claim 27).
- **Share source, shared part.** For a voxel animation: the type, named in its rules section, whose
  model part it shows instead of a model of its own, and that part (claim 30).
- **Art section.** The section of the art database named by a type's image name.
- **Own art section.** For an animation only: the section of the art database named by the
  animation's type name. It is the art section too when the image name equals the type name.
- **Sprite file.** A shape file (public "SHP" format) holding the image frames of an object.
- **Motion file.** The companion file of a voxel model (public "HVA" format), with the same base
  name.
- **Model parts.** A voxel object's *body* model, an optional *turret* model, an optional *barrel*
  model and, for some vehicles, an *alternate body* model.
- **Theater extension, theater letter.** Each theater's three-letter extension and one-letter code
  (Constants, Table 1).
- **Type read.** One reading of a type's rules and art settings, at which its art is built and
  loaded. Every game creates the types of every kind this spec covers afresh, after the game's
  theater is set (claim 24); only tiberiums carry over between games (S-0004 claim 1), and they load
  no art file of their own (claim 32). The rules layers of a game are the base rules, the language
  rules file if a valid or empty one exists, the game-mode layer and the map layer (S-0006). A type has a type read in a rules layer
  when it exists by the time its kind's reading pass runs in that rules layer (S-0004 claim 6) and
  the rules layer has a section for it. A type that a rules layer creates by a reference that S-0004
  claim 6 leaves unread in that rules layer (for example an animation first named by a building,
  vehicle, aircraft or infantry type, or by a late game-wide section) therefore has no type read in
  that rules layer (S-0006 claim 7).
  - **Animations** are read from the art database in every rules layer (S-0006 claim 8), so an
    animation with an own art section has a type read in every rules layer in which it already
    exists when the animation pass runs, whatever that rules layer's own file holds.
  - The game-mode layer is applied in every skirmish, except a game at
    whose start the language rules file fails its check (S-0006 claim 12). A mode whose
    named game-mode file cannot be resolved contributes an empty rules layer (S-0005 claim
    19), which still gives every existing animation a type read.
  - So, under the measurement setup (no language rules file), an animation that the base rules'
    animation list names has **three** type reads in every skirmish, and one more with a valid
    or empty language rules file. An animation
    created later, by such a reference or by a later rules layer's animation list, has fewer.
- **Theater refresh.** A pass over overlays, terrain objects, smudges, buildings and animations that
  rebuilds their theater-dependent art. It runs once in **every** game, after all type reads of that
  game, whether or not the theater differs from the previous game's. Where the refresh sets a kind's
  art, that result is the one seen in play.
- **Letter substitution.** The name change of claim 7: the second character becomes the theater
  letter.
- **Generic fallback.** The retry of claim 8: the second character becomes G.
- **Expansion archive.** As in S-0005: a loose archive named with an expansion number nn. It ranks
  just below the two language archives and above the main expansion archive (S-0005 claim 9).
  Scenarios use one, called the *test expansion archive*, to add or replace files without touching
  stock data. It cannot replace a member at the top level of a language archive (Measurement
  scenarios, Common setup).
- **Name ‖ suffix.** Concatenation of strings. "‖ .ext" means "with the extension ext": a dot and the
  extension are appended. In prose, "image ‖ .ext" is short for image name ‖ .ext.
- **Per-session store** (with its sorts *kept file*, *deferred sprite* and *art read afresh*). S-0005's
  terms (S-0005 claim 22). This spec uses them only in claim 24, to say that it makes no claim about
  them.
- **Cut to n bytes.** A text value longer than n bytes is replaced by its first n
  bytes; a shorter one is unchanged. This is S-0003's text rule (claim 1) with a text limit of
  n. Text limits count input bytes, not decoded Unicode characters. Names are not decoded.
- **Main icon, alternate icon.** A buildable type's two sidebar pictures (claims 22 and 25). The
  sidebar shows one of them for the type.
- **Translucent sprite, translucent-sprite name.** A type's optional secondary (alpha) sprite, and
  the name that the section of claim 1 gives it (claim 23). It is drawn as the type's light-glow
  image (S-0104), not with the translucency codes of S-0109 (animation playback).
- **Infiltrated factory.** A house *has infiltrated a barracks* when its infiltrated-infantry-factory
  flag is set, and *has infiltrated a war factory* when its infiltrated-vehicle-factory flag is set
  (naval vehicles included). These flags result from its spies infiltrating another house's factory,
  subject to the effect selection of S-0510 claim 3. S-0510 claim 5 owns the flags; this spec only
  uses the result.

## Behaviour

This spec covers skirmish only. Network games and campaigns are out of scope.

### Image name and art section

1. **Image name.** A type's image name is the value of the image key in the type's own section of the
   rules database. At a type's first type read in a game the default is the type name. At a later type
   read in the same game, a missing image key keeps the image name already set (claim 10 gives a
   consequence). An empty value counts as missing (S-0003), so the image name is never empty.
   Projectiles differ (claim 26). Values
   longer than 24 bytes are cut to their first 24 bytes. For **animations**, the image key
   is read from the own art section, not from the rules database.
   - Backing: `measurable` (A4 for infantry, A9 for animation image indirection; S-0004 K7 for
     byte rather than decoded-character limits). `unmeasured` for other kinds individually and
     an animation's missing-image default: these scenarios do not isolate those cases.
   - Confidence: high

2. **Art section; animation flags.** All art settings of a type (the theater flag, the new-theater
   flag, the voxel flag, the line-trail settings, building art settings, the icon names) are read from
   its art section, keeping the earlier value where the art section does not set them, except where
   claims 9, 11, 22 and 25 say otherwise. Several types
   can share one art section. For **animations**, the theater flag and the new-theater flag also
   depend on the own art section, and on whether the animation's previous type read in the same
   game found its sprite file (claim 14):

   | Type read of an animation | Each of the two flags after the read |
   |---|---|
   | the first type read of the game, or the previous type read did not find the sprite file | the own art section's value if that section sets the flag; else the art section's value if that sets it; else the value from before the read (not set, at the first read) |
   | the previous type read found the sprite file | the art section's value if that section sets the flag; else the value from before the read |

   Only the immediately previous read counts: a read that follows one that did not find the sprite
   file takes the first row, even if an earlier read had found it.

   Consequence: when the image name differs from the type name and both sections set a flag, but to
   different values, the value in force in play is the own art section's exactly when the game has
   only one type read of the animation or the second-to-last type read of the game did not find the
   animation's sprite file. When that read did find it, the art section's value is in force. When the image name equals the type name, the two
   sections are one and the flags are simply its values.
   (`quirks.anim_flags_follow_held_sprite`)
   - Backing: `measurable` (A9 for the animation flag table, A9 iii for read-count dependence).
     `unmeasured` for retention of every other art setting individually: their consumers need
     separate scenarios; A9 does not establish the general retention assertion.
   - Confidence: high (the table), medium (the number of type reads per game, Terms)

3. **Image keys inside the art database.** An art section can itself hold an image key. It is
   honoured only for **buildings**, where it names the body sprite file (claim 11). For
   **animations** the image key that counts is the one in the own art section (claim 1). An image key
   in the art section named by an animation's image name is never used. For infantry, vehicles and
   aircraft an image key in the art section is ignored: their file names come from the image name of
   claim 1.
   - Backing: `measurable` (A4 for infantry, A9 for animations, A12 part b for buildings).
     `unmeasured` for ignoring an art-section image key on vehicles and aircraft individually:
     no scenario part changes that key for those kinds.
   - Confidence: high

4. **Sprite or voxel model.** Infantry, vehicles and aircraft use a voxel model when their art section's
   voxel flag is set (default: not set), and a sprite file otherwise. A type with the voxel flag set
   never loads a body sprite, even if its voxel model is missing (claim 18). Buildings, animations,
   overlays, terrain objects and smudges always use sprite files for their main art. Buildings can
   add voxel turret parts (claim 20). Projectiles are drawn as a voxel model when their art section's
   voxel flag is set and as a sprite otherwise (claim 27). Particles and particle systems use sprite
   files only, and have no sprite and no model when their art section sets the voxel flag (claims 28,
   29).
   A voxel animation always uses a voxel model (claim 30). These are lookup choices, not a
   promise that a kind renders every loaded part. In particular, an aircraft with the voxel
   flag cleared still looks up sprite art but draws no body (S-0102 claim 21, scenario D34,
   confidence high). Its trailer can still be observed, as in A24.
   - Backing: `measurable` (A5 h for a vehicle's missing voxel body not selecting a sprite,
     A20 c for voxel projectiles, A21 b for particles). `unmeasured` for the remaining
     per-kind sprite/voxel choices: no local part isolates each choice, and particle-system
     lookup has no picture of its own (claim 29).
   - Confidence: high

### Sprite file names

5. **Base sprite name.** The sprite file of an infantry, vehicle, aircraft, overlay or animation type
   is image ‖ .shp, changed by claims 6 to 10. The same holds for particles, particle systems,
   voxel animations and projectiles, with the differences that claims 26 to 30 state.
   - Backing: `measurable` (A4 for infantry, A2 for overlays, A8 for animations, A20 b for
     projectiles, A21 a for particles). `unmeasured` for sprite vehicles and aircraft and the
     auxiliary sprite lookups of particle systems and voxel animations: the proposed pictures
     do not distinguish these individual base-name rules.
   - Confidence: high

6. **Theater flag: theater extension.** For the kinds of claim 5, when the art section's theater flag
   is set (for an animation, the theater flag as claim 2 gives it), the sprite file is image ‖ .E instead, where E is the current theater's extension (for
   example ".tem" in the temperate theater). Letter substitution is not applied to such a name.
   Buildings differ (claim 11).
   - Backing: `measurable` (A2 for overlays). `unmeasured` for the theater-extension choice
     for each remaining kind of claim 5: A2 does not isolate them individually.
   - Confidence: high

7. **Letter substitution.** When it applies (Table 2), letter substitution changes the second
   character of the file name to the current theater's letter, but only if the first character is one
   of G, N, C or Y and the second character is A or T. Letters are compared without regard to case.
   Any other name is left unchanged.
   - Backing: `measurable` (A1, A3)
   - Confidence: high

8. **Generic fallback.** If the sprite file cannot be resolved, the original replaces the second
   character of the file name with G, whatever that character is, and tries once more. This happens
   whether or not any theater rule changed the name, and also for a name built with the theater
   extension. For a one-character image name the second character is the dot, so "Q.shp" is retried
   as "QGshp". If the retry also fails, the type has no sprite (it is not drawn). Terrain objects and
   smudges never get this retry (claim 13), and neither do the theater-flag refreshes of buildings
   and animations (claims 12 and 14).
   (`quirks.art_generic_fallback_any_name`)
   - Backing: `measurable` (A3)
   - Confidence: high

9. **Arctic alternate.** The alternate-arctic-art flag is read from the same section as the image key
   of claim 1: the type's own section of the rules database, or, for an animation, its own art
   section. This is the general rule for animation properties (S-0006 claim 8), not a special case,
   so an animation's flag is the same at every type read of a game. If the flag is set and the
   current theater is snow, the letter A is appended to the image name at the type read (claim 10 gives when
   exactly, subject to the eligibility below), and the sprite name is built from the suffixed name (so image "X" loads "XA.shp"). The
   suffixed name is cut to 24 bytes. The suffixed name then serves as the image name for every
   purpose: the type's own sprite (claim 5), the sidebar icons (claims 22 and 25), and the art
   settings that other specs read from the art section, which come from the art section named by
   the suffixed name. For these non-projectile types, the exception is the theater, new-theater, voxel and line-trail
   settings *of the read that adds the suffix*: they come from the art section of the name as it
   was before that read added the A. A building's convert-to-overlay reference and a sprite
   aircraft's trailer settings use the suffixed art section in that same read (claim 33,
   S-0004 claim 14).
   - The suffix applies to sprite-body infantry, vehicles and aircraft, and to overlays, buildings,
     terrain objects and smudges. For buildings, terrain objects and smudges, their own naming rules
     (claims 11 to 13) use the suffixed image name. It also applies to particles, particle systems
     and voxel animations whose art section does not set the voxel flag (claims 28 to 30; a voxel
     animation's model is then named by the suffixed name). Projectiles follow claim 27 instead.
   - It also applies to animations, at the type reads after which the theater flag is set (claim
     2), but has no visible effect there while the theater flag stays set after the last type read,
     because their art in play is then named by type name (claim 14).
   - It never applies to the other kinds' types with the voxel flag (infantry, vehicles, aircraft,
     particles, particle systems, voxel animations), nor to an animation at a type read after which
     its theater flag is not set.
   - Backing: `measurable` (A6) for the suffix and the sprite; `unmeasured` for the section an
     animation's flag comes from. While the theater flag stays set the suffix is invisible (above),
     and a test that clears the flag at the last read would rest on claim 2's alternation and the
     game's read count as well, so the source section cannot be told apart cleanly on screen.
     The remaining per-kind suffix eligibility, suffix truncation and art-section selection are
     `unmeasured`: A6 tests infantry sprite, icon and same-read voxel selection only; no part
     isolates each other setting or kind. Claim 33 and A24 separately cover reference sections.
   - Confidence: high (the suffix and the sprite, the section an animation's flag comes from),
     medium (which settings come from which section).

10. **When the arctic suffix is added.** Only snow games are affected. At each type read after which
    the alternate-arctic-art flag is set and claim 9 makes the type eligible, an A is appended to the image name as that read leaves it,
    unless the flag was also set after the type's previous type read in the same game. The first type
    read of a game counts as following a read with the flag not set. A read after which the flag is
    not set appends nothing and removes nothing. So an A is appended exactly at the reads where the
    flag goes from not set to set, and at the first read if the flag is set there. Whether the
    previous read appended an A does not matter. Observable consequences within one game:
    - A later type read (game-mode file or map override section, S-0006) that **has a non-empty image key**
      replaces the image name with that key's value, even when it repeats the same text as the
      earlier rules layer's value, and the result gets **no** A while the flag stays set. On snow the type then
      shows the non-arctic file, and its later art settings come from the unsuffixed art section.
      Further type reads without an image key do not bring the A back while the flag stays set
      (for example a game-mode override section with an image key, then a map override section
      without one: the name stays unsuffixed).
    - A later type read **without an image key** keeps the name as it stands: the suffixed name if no
      image key has replaced it.
    - If the flag is first set by a later type read (the base rules leave it unset and an override
      section sets it), that read appends the A.
    - If one later read clears the flag and a further read sets it again, a second A is appended to
      the name as it then stands ("X" becomes "XAA" when neither read has an image key).
    - **Animations.** An animation's flag comes from its own art section and is the same at every
      type read (claim 9). The suffix is applied only at reads after which the theater flag is set,
      and the state "flag set after the previous read" changes only at such reads. So on snow the A
      is appended at most once per game: at the first type read of the game after which the
      animation's theater flag is set. If its own art section has an image key, every later type
      read replaces the name with that key's value (claim 1) and no A is appended again: the name
      loses the A for the rest of the game (first bullet above). Without an image key the suffixed
      name stays. Either way, while the theater flag stays set after the last read, the art seen in
      play is named by the type name (type name ‖ .E, else type name ‖ .shp, claim 14), so the difference shows only in the art section that later
      reads take their settings from (claim 2).

    Every game starts again from new types (claim 24), so nothing carries over to the next game.
    (`quirks.arctic_suffix_on_flag_turning_on`)
    - Backing: `measurable` (A7); the animation bullet is `unmeasured`, for the reason given under
      claim 9.
    - Confidence: medium

### Buildings

11. **Building sprite names.** A building's body sprite name is built from its art section's own image
    key if that key is set, and from the image name otherwise. It takes the theater extension if the
    theater flag is set, and ".shp" otherwise. **Letter substitution is then applied using the current
    theater whether or not the new-theater flag is set.** If the file cannot be resolved, the generic
    fallback is tried. The building's other sprites, which the art section names (build-up, deploying,
    roof-deploying, door, under-door, under-roof-door, rubble, special overlay and bib sprites), are
    each name ‖ .shp with letter substitution and the generic fallback, never with the theater
    extension at a type read (claim 12 gives the build-up sprite's name at the theater refresh).

    Name lengths and later reads:
    - The art section's own image key and the eight names other than the build-up name are cut to
      63 bytes. These are byte limits, not decoded-character limits. The image key is read afresh at every type
      read from the art section as it is then: when that art section does not set it, that read uses
      the image name, whatever an earlier read's art section had set.
    - The build-up name is cut to **15** bytes, not decoded characters. A type read whose art
      section does not set it, or sets it empty, keeps the build-up name of the earlier read.
    - **The eight other sprites at a later type read** (deploying, roof-deploying, door,
      under-door, under-roof-door, rubble, special overlay, bib). Each is decided on its own, by the
      art section of that type read:

      | That art section's name for the sprite | The building's sprite after the read |
      |---|---|
      | not set, or set empty | unchanged: the sprite an earlier type read of the game gave it, or none if no earlier read gave one |
      | set, and the name or its generic fallback resolves | the file found, replacing any earlier sprite |
      | set, and neither resolves | none, even if an earlier type read had given one |

      An empty value thus behaves like an absent key, as for the build-up name. Nothing else in a
      game takes these sprites away again. A new-theater building's refresh (claim 12) reads the
      same art section again and gives the same result.
    (`quirks.building_substitution_ignores_flag`)
    - Backing: `measurable` (A1 and A12 parts b, c and e for the body sprite, A2 and A12 parts d and
      e for the build-up sprite, A19 for the bib sprite at a later type read). `unmeasured`
      for the seven other secondary sprite roles and their 63-byte limits, and generic fallback
      on secondary sprites individually: the bib test does not expose each of those roles.
    - Confidence: high (names and substitution, the later-read table), medium (the 63- and
      15-byte cuts)

12. **Building art at the theater refresh.** At the theater refresh of every game, a building with the
    theater flag reloads its body sprite as image ‖ .E and its build-up sprite as build-up name ‖ .E.
    This refresh ignores the art section's own image key, applies no letter substitution and has no
    generic fallback. If the body file cannot be resolved the building has no body sprite, and if the
    build-up file cannot be resolved it has no build-up sprite, even if the type read of claim 11 had
    found one. A building with the new-theater flag (and not the theater
    flag) rebuilds all its sprites exactly as in claim 11, which gives the same names again. A
    building with neither flag keeps the result of claim 11.
    (`quirks.theater_refresh_building_no_fallback`)
    - Backing: `measurable` (A2 for the new-theater case, A12 for the theater-flag body case).
      `unmeasured` for the theater-flag build-up replacement and its no-fallback rule: A12
      does not isolate that sprite, and the other scenarios leave the theater flag cleared.
    - Confidence: high

### Terrain objects, smudges, animations, overlays

13. **Terrain objects and smudges.**
    - A **terrain object without the theater flag** loads image ‖ .shp when the type is read. There
      is no generic fallback and no letter substitution, and the theater refresh does not touch it.
    - A **terrain object with the theater flag** loads nothing when the type is read. At the theater
      refresh it loads **type name** ‖ .E: the type name, not the image name. There is no generic
      fallback.
    - A **smudge without the theater flag** loads image ‖ .shp when the type is read, with no
      fallback, and the refresh does not touch it.
    - A **smudge with the theater flag** loads image ‖ .E when the type is read, and the theater
      refresh then replaces it with **type name** ‖ .E, with no fallback. If type name ‖ .E cannot be
      resolved the smudge has no sprite, even if image ‖ .E existed.

    The general rule of claim 5 does not apply to these two kinds.
    (`quirks.theater_art_uses_type_name`)
    - Backing: `measurable` (A2, A11, A13 for final visible names and terrain's lack of generic
      fallback). `unmeasured` for whether theater-flag terrain loads nothing at a type read,
      the smudge's transient image-named load and smudge-specific fallback failure: no screen
      result precedes refresh, and A11 c tests terrain only.
    - Confidence: high

14. **Animations.** At each type read of an animation, its sprite is loaded again, using the flags in
    force after that read (claim 2):
    - without the theater flag: image ‖ .shp, with the generic fallback and with **no** letter
      substitution, even when the new-theater flag is set.
    - with the theater flag: image ‖ .E with the generic fallback (claim 6).

    At the theater refresh of every game:
    - an animation with the theater flag reloads **type name** ‖ .E, and if that cannot be resolved,
      type name ‖ .shp, with no generic fallback.
    - an animation with the new-theater flag (and not the theater flag) reloads image ‖ .shp with
      letter substitution and the generic fallback.
    - other animations keep the sprite from their last type read.

    So in play a new-theater animation shows its theater-substituted sprite: the refresh's result is
    the one seen, not the type read's.
    (`quirks.theater_art_uses_type_name`)
    - Backing: `measurable` (A8, A13 for refresh results; A9 for type-read resolution affecting
      subsequent flags). `unmeasured` for individual generic-fallback cases at animation type
      reads: the final refresh hides the intermediate picture and no local part isolates them.
    - Confidence: high

15. **Overlays.** When an overlay type is read, claims 5 to 8 apply. At the theater refresh of every
    game, an overlay with the theater flag reloads image ‖ .E with the generic fallback, and one with
    the new-theater flag reloads image ‖ .shp with letter substitution and the generic fallback.
    Both give the same names as the type read.
    - Backing: `measurable` (A2)
    - Confidence: high

### Volumetric models and companion files

16. **Names of the voxel parts.** For a vehicle, aircraft, infantry or projectile type with the voxel
    flag and image name I (for a projectile, the name of claim 27), the parts are looked for under
    these base names. A projectile counts as "not a vehicle" in the table. Each part is a model file base ‖ .vxl
    together with a motion file base ‖ .hva:

    | Part | Base name | Looked for when |
    |---|---|---|
    | body | I | always |
    | turret | I ‖ "TUR" | the type is not a vehicle, or is a vehicle with the turret flag and without the numbered turrets below |
    | barrel | I ‖ "BARL" | same as turret |
    | turret k (k = 0 … n − 1) | I ‖ "TUR" for k = 0, I ‖ "TUR" ‖ k for k ≥ 1 (k in decimal) | a vehicle with the turret flag, a multiple-turret count n ≥ 1, and no gatling flag |
    | barrel k | I ‖ "BARL" for k = 0, I ‖ "BARL" ‖ k for k ≥ 1 | as for turret k |
    | alternate body | I ‖ "W" | a vehicle without the turret flag whose type name is exactly "APC" (upper case) |
    | alternate body | I ‖ "WO" | a vehicle without the turret flag whose rules section sets the no-spawn-alternate flag. It takes the place of the "W" body if both apply. |

    A vehicle without the turret flag has no turret or barrel part. When an alternate body is drawn
    instead of the body is a rendering question (Open questions).
    (`quirks.apc_water_body`)
    - Backing: `measurable` (A5 for sampled vehicle parts and the exact APC-name restriction).
      `unmeasured` for non-vehicle parts, decimal numbering beyond the sampled turret 1, the
      gatling exception and WO precedence over W: A5 does not isolate those individual rules.
    - Confidence: high

17. **Multiple-turret count.** The rules count n of claim 16 selects n turret models. The original
    supports a multiple-turret count of at most 18. With a count above 18 its behaviour is
    *unspecified*. Our engine treats such a count as 18, and the importer warns.
    - Backing: `unmeasured` (no scenario exercises the 18-part boundary, and above it the
      original's result is unspecified, so there is no stable comparison for excessive counts).
    - Confidence: high

18. **Missing or broken voxel parts.**
    - The body model is required. If I ‖ .vxl cannot be resolved, the type has no voxel art and,
      because the voxel flag is set, no sprite either: it is not drawn.
    - Wherever a model file is found, its motion file of the same base name is loaded too. If that
      motion file cannot be loaded, or if any found model or motion file is invalid, the voxel art of
      the type is discarded and the type is not drawn. This covers the body, the turret, the barrel,
      the alternate bodies and the numbered turrets. All of this claim applies to projectiles with
      the voxel flag too (claim 27). A voxel animation differs (claim 30).
    - A missing single turret, single barrel, alternate body or numbered barrel is fine: the part is
      simply absent.
    - A missing **numbered turret** k < n (claim 16) discards the voxel art, whatever the other
      numbered turrets are.
    - Whenever the voxel art is discarded, nothing of the type is drawn.
    - A numbered barrel that exists but is invalid gives an *unspecified* result in the original. Our
      engine discards the voxel art, as for any other invalid part.
    (`quirks.voxel_part_failure_discards_all`)
    - Backing: `measurable` (A5 for missing body, optional barrel, numbered turret/barrel,
      alternate body and missing/invalid motion files on the tested vehicles). `unmeasured`
      for remaining optional-part failures, invalid model files, each non-vehicle kind and
      invalid numbered barrels: these cases are not isolated, or have unspecified results.
    - Confidence: high (missing body, missing numbered turret, missing optional parts, invalid motion
      file), medium (a missing motion file next to a found model, A5 part i; that nothing is drawn
      after a discard)

19. **Sprite body with a voxel turret.** A vehicle that uses a sprite body (voxel flag not set) and has
    the turret flag also looks for a voxel turret I ‖ "TUR" and barrel I ‖ "BARL" (single parts only,
    no numbered parts), provided its body sprite loaded. Each part loads on its own: a missing or
    invalid part never removes the body sprite or the other part, and nothing is discarded.
    - Backing: `measurable` (A14) for the body sprite staying with valid parts and an invalid
      turret motion file. `unmeasured` for missing parts and individual barrel failures:
      A14 does not vary them.
      `unmeasured` for the lookup of the voxel turret and barrel, which shows only in how they are
      drawn on a sprite body (S-0105), so A14 records whether a turret appears without an expected
      result.
    - Confidence: medium

20. **Building voxel turrets.** A building whose rules set the turret-animation-is-voxel flag or the
    barrel-animation-is-voxel flag takes its voxel turret parts from its turret-animation name T
    (rules). Say that T has a *late "TUR"* when the letters "TUR" (any case) occur in T starting at
    its fifth character or later. An occurrence that starts within the first four characters does not
    count, so a name such as "TURX" or "ATUR" has no late "TUR". Let P be T up to, and not including,
    its first late "TUR" (anything from that "TUR" on is dropped).

    | T has a late "TUR" | Barrel-animation-is-voxel flag | Turret model | Barrel model |
    |---|---|---|---|
    | yes | set or not | T | P ‖ "BARL" |
    | no | set | none | the building's voxel barrel file name (rules) |
    | no | not set | none | **T itself** |

    Each model loads with its motion file of the same base name. A missing file leaves that part
    absent. Unlike
    vehicle voxels (claim 18), nothing is discarded here: each part loads or not on its own.
    (`quirks.building_turret_without_tur_in_barrel_role`, `quirks.building_tur_search_skips_four`)
    - Backing: `measurable` (A10 for the four supplied-name cases). `unmeasured` for missing
      parts, repeated late occurrences, case-insensitive matching and other early positions:
      no scenario part isolates these cases. Model role placement remains low-confidence
      until rendering scenarios distinguish it.
    - Confidence: high (which files load), low (what the barrel role means on screen, S-0105)

21. **No separate shadow files.** No object kind loads a separate shadow file. A sprite's shadow is
    drawn from image frames inside the same sprite file, and
    a voxel shadow is derived from the model (S-0105). Which image frames hold the shadow is
    given for infantry by S-0107 and for buildings by S-0108; for animations it is given by
    S-0109 (animation playback).
    For overlays, terrain objects, smudges, projectiles and particles shadow-frame selection is outside
    this spec; this claim says only that no separate file is loaded for them. It holds for every kind in
    this spec.
    - Backing: `unmeasured` (A15 tests an infantry sprite only. The universal claim that no kind
      loads a separate shadow file has no observable measurement proposal here: the absence of a
      separate load does not itself produce a visible result, and the draft gives no established
      separate-file name or other observable to test for the remaining kinds.)
    - Confidence: high

### Other art files of a type

22. **Main icon.** A buildable type's main icon is the art section's icon name ‖ .shp, with the icon
    name cut to 24 bytes. There is no theater handling and no generic fallback. If that file
    cannot be resolved (or the name is empty), the hard-coded placeholder "XXICON.SHP" is used. The
    icon is set at the type's first type read in a game and is not replaced by later type reads in the
    same game: a game-mode file or map override section cannot change it, not even by changing the
    image name. When the sidebar shows the main icon is claim 25. S-0411 claim 16 owns
    the chosen icon file's image frame and content placement.
    (`quirks.main_icon_first_read_only`)
    - Backing: `measurable` (A16 for placeholder, first-read freeze and byte limit).
      `unmeasured` for empty icon names and absence of theater handling or generic fallback:
      A16 does not supply competing files for those lookups.
    - Confidence: high (placeholder, first-read freeze), medium (the 24-byte cut)

23. **Translucent sprite.** If a type's own section sets a translucent-sprite name, the file
    translucent-sprite name ‖ .shp is loaded, with no theater handling and no generic fallback. The
    name is cut to its first 24 bytes. The section is the one of claim 1: the type's section of
    the rules database, or, for an animation, its own art section (so a rules layer cannot change an
    animation's translucent sprite, S-0006 claim 8). The loaded sprite is the type's light-glow image
    of S-0104 (Terms, claim 15) and S-0110 (claim 6), which changes the light-alpha values under it
    instead of being painted. A building shows it, and so does an animation, from its creation. This
    spec makes no claim about which other kinds show it, or when. It is unrelated to an animation's
    translucency codes (S-0109), which only make the animation's own sprite see-through. Where the glow is placed and how it writes
    the values is S-0104's and S-0110's.
    - Backing: `measurable` (A17; part d for an animation's translucent sprite).
      `unmeasured` for missing-name retention across layers and every remaining type kind:
      the proposed glow tests exercise buildings and animations only.
    - Confidence: medium (high for the section an animation's key comes from; medium that an
      animation's glow shows, since the glow's own drawing is S-0104's)

24. **When names are built.** Every game creates the types of the kinds this spec covers afresh after
    the game's theater is set (Terms, *type read*).
    Sprite and voxel names are built and loaded at the type reads of the game (base rules, language
    rules file if valid or empty, game-mode layer except in a game at whose start the language
    rules file fails its check, map layer; Terms,
    *type read*), and the theater refresh follows after all of them. The refresh does not touch
    projectiles, particles, particle systems, voxel animations or superweapon icons: for them the
    last type read's result is the one seen in play. The observable consequences
    of this order are listed under State and ordering. All names resolve through
    S-0005, so they match without regard to letter case. Which of these loads go through the
    per-session store, and as which sort (kept file, deferred sprite or art read afresh), is S-0005
    claim 22; this spec makes no claim about it.
    - Backing: `measurable` (A1, A4, A8 for sampled game-start and layered consequences).
      `unmeasured` for rebuilding every kind individually and refresh exclusions for every
      auxiliary kind: no scenario independently contrasts these for each listed kind.
    - Confidence: high

### Alternate sidebar icon

25. **Alternate icon.** A buildable type may also have an alternate icon, named by its art section's
    alternate icon name.
    - **Loading.** The file is alternate icon name ‖ .shp, with the name cut to 24 bytes, no
      theater handling, no generic fallback and **no placeholder**. A type read loads it only if the
      type has no alternate icon yet and its art section sets a non-empty alternate icon name. Once
      an alternate icon is loaded, later type reads of the game never replace it. Unlike the main
      icon (claim 22), a later type read *can* give the type an alternate icon when the earlier
      reads did not, for example when a map override section changes the image name to an art
      section whose alternate icon file resolves.
    - **Which icon the sidebar shows.** The sidebar shows the alternate icon of a type exactly when
      all of these hold, and the main icon otherwise:
      - the type has an alternate icon (it loaded),
      - the type's trainable flag is set (rules), and
      - the local player's house has infiltrated a barracks and the type is an infantry type, or it
        has infiltrated a war factory and the type is a vehicle type or a building type whose
        undeploy-into vehicle is set (rules).

      Aircraft types and other building types always show the main icon. The choice is made each
      time the sidebar shows the icon, so it changes as soon as the infiltration happens.
      S-0411 claim 16 owns the chosen file's image frame and content placement.
    (`quirks.alt_icon_kept_once_loaded`)
    - Backing: `measurable` (A18 for infiltration, trainability, missing files and late first
      loading). `unmeasured` for retaining an already loaded alternate icon, its byte limit,
      theater handling, generic fallback and empty names: no part varies those cases.
      The exact tick at which the sidebar choice changes is also `unmeasured`: screenshots
      after infiltration do not isolate same-tick display.
    - Confidence: high (the loading rules, the three conditions), medium (that the choice follows the
      infiltration at once)

### Projectiles, particles, particle systems, voxel animations, superweapons

26. **Projectile image name.** After a type read of a projectile, its image name is the value of the
    image key in that read's section of the projectile (cut to 24 bytes), and **empty** when that
    section has no image key or an empty one. Unlike claim 1, there is no default of the type name and
    nothing carries over from an earlier type read. A projectile with an empty image name has no
    sprite and no model in our engine; its translucent sprite still follows claim 23. For an empty
    image name, the original attempts empty-base names such as ".shp". Its result when such files
    are supplied is not established here. Suppressing these lookups is a documented departure.
    Consequences:
    - A non-invisible projectile without an image value shows no body art when no empty-base file
      resolves.
    - A later type read of a projectile (a game-mode file or map override section for it, S-0006)
      that has no image key leaves it without art for the rest of the game, even if the base rules
      gave it an image key. The importer warns when such a section has no image key while an earlier
      rules layer gave one.
    (`quirks.projectile_image_afresh`)
    - Backing: `measurable` (A20 a and b; S-0004 K7 for the byte limit; A24 for empty values).
      `unmeasured` for original lookups when empty-base files exist: no part supplies them,
      and their outcome is not established. Our engine's suppression is a design departure.
    - Confidence: high

27. **Projectile art.** Let I be a projectile's non-empty image name (claim 26). Its art section is
    the one named by I. Its trailer animation and trailer delay use this section before any arctic
    change to the drawing name (claim 33; S-0004 claim 14). Thus selecting an arctic voxel model
    does not select the suffixed art section's trailer animation or delay: the trail still uses
    the unsuffixed section's settings. The theater flag, the new-theater flag, the voxel flag and
    the line-trail settings follow claim 2 and the exception in claim 9.
    - **Without the voxel flag:** the sprite is the one claims 5 to 8 give an infantry sprite (found
      and subst as defined under Formulas): found(I ‖
      .E) with the theater flag, found(subst(I ‖ .shp)) with the new-theater flag, found(I ‖ .shp)
      otherwise. On snow the alternate-arctic-art flag has **no** effect on such a projectile: its
      sprite is always built from I, never from I ‖ "A".
    - **With the voxel flag:** the projectile is drawn as a voxel model with the parts of claim 16 for a
      type that is not a vehicle (body I, optional turret I ‖ "TUR", optional barrel I ‖ "BARL"), and
      claim 18 decides when the voxel art is discarded. A sprite is never drawn for it, whatever
      files exist.
      - On snow, with the alternate-arctic-art flag set after the read, the model is named by
        I ‖ "A" instead of I at the type reads where claim 10 appends an A (the first type read of
        the game, or a read at which the flag goes from not set to set). Because I is taken afresh at
        every type read (claim 26), the A lasts only until the projectile's next type read: a later
        type read with the flag still set names the model by I again.
    - **Invisible projectiles** are never drawn, whatever art files exist, so their art is never
      seen and this spec gives no result for it. An implementation need not look it up.
    - The theater refresh does not touch projectiles (claim 24).
    - Backing: `measurable` (A20 b to f for image selection, invalid body motion, first arctic
      transition and invisibility; A24 for the unsuffixed trailer). `unmeasured` for projectile
      theater and generic names, optional parts and loss of the arctic suffix on a subsequent
      read: no part isolates those rules for a projectile.
    - Confidence: high (sprite names, voxel parts), medium (the arctic results)

28. **Particles.** A particle's image name follows claim 1 (its rules section; default the type
    name; a later type read without an image key keeps it). Its sprite follows claims 5 to 10 like an
    infantry sprite: theater extension, letter substitution with the new-theater flag, generic
    fallback and arctic suffix. A particle whose art section sets the voxel flag has no sprite and
    no model; its translucent sprite still follows claim 23. The theater refresh does not touch
    particles.
    - Backing: `measurable` (A21 for generic fallback and voxel-flag suppression).
      `unmeasured` for the other particle-specific name rules, default and retention:
      A21 does not vary theater, arctic state or omitted image values.
    - Confidence: high

29. **Particle systems.** A particle system's art is looked up exactly as a particle's (claim 28),
    and its translucent sprite as in claim 23. This spec makes no claim that a particle system draws
    that sprite: what a particle system shows is the particles it emits, which belong to the particle
    spec. An implementation therefore needs only to record the name.
    - Backing: `unmeasured`. The lookup produces no picture of its own, so there is nothing on screen
      to compare.
    - Confidence: medium

30. **Models of voxel animations.** A voxel animation's image name I follows claim 1 (its rules section,
    default the type name, kept when a later type read has no image key, cut to 24 bytes).
    - **Own model.** At each type read the model is the file I ‖ .vxl, read afresh. **No motion file
      is loaded** for it, and there are no turret, barrel or alternate-body parts, no theater handling
      and no generic fallback. The arctic suffix of claims 9 and 10 applies to I only when the
      voxel animation's art section does not set the voxel flag; the model is then named by the
      suffixed name. A sprite named as in claim 28 is also looked up in that case; this spec makes no
      claim that it is drawn.
    - **Shared part.** If the type read's section sets any of the three share flags (body, turret,
      barrel), the voxel animation loads no model of its own. It shows a part of its share source
      instead, with that part's motion file:
      - The share source is a type of one of the kinds infantry, vehicles, aircraft and buildings
        whose type name matches the named value, without regard to letter case. When the value
        matches types of more than one of these kinds, an infantry type is the share source in
        preference to a vehicle type, a vehicle type in preference to an aircraft type, and an
        aircraft type in preference to a building type. A value that matches no type, or no value,
        gives no share source.
      - The part is the one whose share flag is set. When more than one share flag is set, the body
        takes precedence over the turret, and the turret over the barrel. For a building, the turret
        and barrel are the models of claim 20, and it has no body model.
      - Without a share source, or if the source lacks that part (or its voxel art was discarded,
        claim 18), the voxel animation has no model.
      - In each rules layer, the part taken is the one the source has after its own type read in
        that same rules layer, not the one it had before that read (by the reading order of S-0004
        claim 6).
    - **Later type reads.** The share flags and the share source are read afresh at every type
      read, with "not set" and "none" as defaults. A later type read that sets share flags again
      takes the part named then, for example another source's turret. A later type read that sets
      none of the share flags, after an earlier read of the game had shared a part, gives an
      *unspecified* result in the original: the voxel animation turns to its own model, and the
      shared part, which the source type still uses, is left in an unspecified state, so what the
      source shows for that part is unspecified too. Our engine shows the voxel animation's own
      model I ‖ .vxl, leaves the source's part intact, and the importer warns about such a section.
      (`quirks.voxel_anim_share_afresh`)
    - **Departures.** If the source type has a type read in a later rules layer in which the
      voxel animation has none, the original's result is *unspecified*. Our engine then shows the
      source's part as it stands after the source's last type read, and the importer warns. What the
      original draws when I ‖ .vxl cannot be resolved or is invalid is not established here (Open
      questions); our engine draws nothing for that voxel animation, and the importer warns.
    - The theater refresh does not touch voxel animations.
    - Backing: `measurable` (A22) for the own model, the shared part, the share priorities (A22
      parts d and e) and a later read that shares again; `unmeasured` for a later read that drops
      the share flags, because the original's result there is unspecified and a measurement would
      record chance, not a rule. A22 part e also measures which kind wins when a share-source name
      matches more than one kind. S-0004 claim 13 marks this priority measurable and cites
      S-0007's A22 part e, which distinguishes the winning kind through the shared-model outcome.
      Remaining own-name defaults, arctic eligibility, missing share sources or parts, building
      part sharing and same-layer source-read selection are `unmeasured`: no scenario part
      isolates those individual rules, and late source rereads have unspecified original results.
    - Confidence: high (the file name, the share order and priority), medium (no motion file, the
      afresh share keys)

31. **Superweapon sidebar icon.** A superweapon's sidebar icon is c ‖ .shp, where c is the sidebar
    icon name in the superweapon's rules section, cut to 24 bytes. Its default is the type
    name, and a later type read without the key keeps the name (an empty value counts as missing,
    S-0003). There is no theater handling and no generic fallback. If the file cannot be resolved,
    the placeholder "XXICON.SHP" is used, as for the main icon (claim 22). Unlike the main icon, the
    superweapon icon is built again at **every** type read, so a game-mode file or map override
    section can change it.
    - Backing: `measurable` (A23 for replacing, retaining and missing icon names).
      `unmeasured` for type-name default, empty names, byte limit and absence of theater
      handling or generic fallback: A23 does not distinguish those cases.
    - Confidence: high

32. **Kinds without art files of their own.** Weapons, warheads and tiberiums load no art file of
    their own, through an image name or any other name. Weapons and warheads only name other types
    (animations, particle systems, projectiles), whose art follows those types' own claims. A
    tiberium's image key is a number that selects its overlays, not a file name (S-0401
    covers it).
    - Backing: `unmeasured`. A load that does not happen leaves nothing to see, and these kinds have
      no picture through which a file could show.
    - Confidence: high

33. **Art sections for references and trailers.** A building's convert-to-overlay reference and
    an aircraft's trailer animation and trailer delay come from the art database, never their
    rules section. They use the image name in force in that type read, including any snow suffix
    that claims 9 and 10 add in that read. Thus a sprite aircraft can have a different trailer on
    snow, and a building can create a different overlay type there. A voxel aircraft does not
    acquire the suffix. S-0004 claim 14 owns the registration consequences of these references.
    - A projectile's trailer animation and trailer delay also come from the art database. Their
      section is named by the non-empty image value supplied by that read's rules section,
      cut to 24 bytes, before any snow suffix changes its drawing name. Even a voxel projectile
      whose model uses an arctic name takes its trailer settings from the unsuffixed section.
      There is no fallback to its type name. If the current read has no image value, or an empty
      one, neither trailer setting is read: both keep their earlier values, even though the
      projectile loses its own picture under claim 26. Losing its picture does not clear its trail.
    - A missing or empty trailer reference keeps the earlier reference, initially none. A reserved
      no-type name clears it, by S-0004. A missing or empty delay keeps the earlier delay, initially 3 ticks.
      These reads require the type's rules section in the current layer, as S-0004 claim 14 says.
      Trailer timing during projectile flight is S-0306 claim 16, not an art-naming rule.
    - Backing: `measurable` (A24 for trailer section selection, snow and retained projectile
      settings; S-0004 K2 and K7 for the building reference and image-byte limit). Retaining or
      clearing a building's held reference remains `unmeasured`, as S-0004 claim 14 explains:
      already registered overlays do not disappear when the reference changes.
    - Confidence: high

## Formulas

All name quantities are byte strings: there is no arithmetic, so no rounding rule applies and no
floating point is involved. Letters are compared without regard to case.

Notation:
- I is the image name *after* claims 1, 9 and 10 (so it already carries any arctic suffix), and
  "type name" is the type's type name. The projectile formula below overrides this definition of I.
- E is the current theater's extension and L its letter (Table 1). A theater is always set when these
  formulas apply (claim 24).
- For a file name f, f₁ and f₂ are its first and second characters, and f⟨2 → x⟩ is f with its second
  character replaced by x.

- ∅ stands for "no file".

> subst(f) = f⟨2 → L⟩ when f₁ is one of G, N, C, Y and f₂ is A or T; otherwise subst(f) = f.
> generic(f) = f⟨2 → G⟩, whatever f₂ is (for a one-character image name, f₂ is the dot).
> found(f) = f when f resolves; = generic(f) when f does not resolve and generic(f) does; = ∅
> otherwise.

**Infantry, vehicle and aircraft sprite** (voxel flag not set), at each type read:

| Flags after the type read | Sprite file |
|---|---|
| Theater flag set | `found(I ‖ .E)` |
| Theater flag not set and new-theater flag set | `found(subst(I ‖ .shp))` |
| Neither flag set | `found(I ‖ .shp)` |

**Building body sprite**, at each type read: Let J be the art section's own image key when it is
set, and I otherwise. If the theater flag is set, the sprite file is `found(subst(J ‖ .E))`; otherwise
it is `found(subst(J ‖ .shp))`.

**Building body sprite at the theater refresh**, when the theater flag is set (claim 12): the
sprite file is I ‖ .E if that name resolves, and ∅ otherwise.

**Animation sprite**, at each type read: With the theater flag set, the sprite file is
`found(I ‖ .E)`. Without that flag, it is `found(I ‖ .shp)`, with no letter substitution.

**Animation sprite at the theater refresh:**

| Flags after the last type read | Sprite at refresh |
|---|---|
| Theater flag set | Use (type name) ‖ .E if it resolves. If it does not resolve, use (type name) ‖ .shp if that resolves; otherwise there is no sprite (∅). |
| Theater flag not set and new-theater flag set | `found(subst(I ‖ .shp))` |
| Neither flag set | Keep the sprite from the last type read. |

**Terrain object and smudge sprite** (no generic fallback anywhere): When the theater flag is not
set, the type read loads I ‖ .shp if it resolves, and otherwise loads no sprite (∅). When the
theater flag is set, the theater refresh loads (type name) ‖ .E if it resolves, and otherwise
loads no sprite (∅).

**Projectile** (claims 26, 27): Let K be the image key in the projectile's section for this type
read, cut to 24 bytes. I is K when the key is non-empty, and otherwise is empty.

| Condition | Art for this type read |
|---|---|
| I is empty, or the projectile is invisible | No body art in our engine (claim 26). |
| I is non-empty and the voxel flag is not set | The sprite given by the infantry-sprite rule above, using I and never I ‖ "A". |
| I is non-empty and the voxel flag is set | The model parts of claim 16, named from I; at a snow-game read where claim 10 appends A, name them from I ‖ "A" instead. |

**Particle and particle system** (claims 28, 29): the infantry sprite formulas above, with claim 1's
image name; no body art when the art section's voxel flag is set.

**Model of a voxel animation** (claim 30), at each type read:

| Share flags | Model |
|---|---|
| At least one share flag is set | Use the share source's part named by the highest-priority set flag (body before turret, turret before barrel), together with its motion file. If there is no share source or it lacks that part, there is no model (∅). |
| No share flag is set | Use I ‖ .vxl. No motion file is loaded, and there is no fallback. |

**Superweapon icon** (claim 31), at every type read: Let s be the sidebar icon name cut to 24
bytes. The icon file is s ‖ .shp when that name resolves; otherwise the placeholder file
XXICON.SHP is used.

**Icons** (claims 22 and 25): Let c be the main icon name and a the alternate icon name, each cut
to 24 bytes. At the first type read only, the main icon is c ‖ .shp if c is non-empty and the
file resolves; otherwise it is XXICON.SHP. At each type read while no alternate icon is loaded,
the alternate icon becomes a ‖ .shp if a is non-empty and the file resolves; otherwise no
alternate icon is loaded (∅). Once loaded, the alternate icon is not replaced.

subst never changes a name whose second character is not A or T, and an extension never affects the
first two characters of a name of two or more characters. So subst on J ‖ .E changes only names that
start with a matching pair.

## Constants

**Table 1 — theaters** (engine constants, shared with S-0005).

| Theater | Extension E (text, 3 characters) | Letter L (character) |
|---|---|---|
| temperate | TEM | T |
| snow | SNO | A |
| urban | URB | U |
| desert | DES | D |
| new urban | UBN | N |
| lunar | LUN | L |

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| image name length | 24 | bytes | longer values are cut to the first 24 |
| translucent-sprite name length | 24 | bytes | as above |
| building art-section image key length | 63 | bytes | longer values are cut (claim 11) |
| building secondary sprite name length (all but build-up) | 63 | bytes | longer values are cut (claim 11) |
| build-up sprite name length | 15 | bytes | longer values are cut (claim 11) |
| icon name length, alternate icon name length | 24 | bytes | longer values are cut (claims 22, 25) |
| projectile image name length | 24 | bytes | longer values are cut (claim 26) |
| superweapon sidebar icon name length | 24 | bytes | longer values are cut (claim 31) |
| initial trailer delay | 3 | ticks | aircraft and projectiles, claim 33 |
| substitution first letters | G, N, C, Y | characters | case-insensitive |
| substitution second letters | A, T | characters | case-insensitive |
| generic fallback letter | G | character | replaces the second character |
| arctic suffix | A | character | appended to the image name |
| turret-part suffix | TUR | text | |
| barrel-part suffix | BARL | text | |
| alternate body suffixes | W, WO | text | |
| type name with the "W" body | APC | text | exact, upper case |
| late "TUR" start | 5 | character position, counted from 1 | earliest position that counts (claim 20) |
| multiple-turret count limit | 18 | count | largest count the original supports (claim 17) |
| placeholder icon | XXICON.SHP | file name | |
| sprite, model and motion extensions | SHP, VXL, HVA | text | |

## State and ordering

**Table 2 — which theater rule applies, by kind and moment.**

| Kind | At each type read (theater already set) | At the theater refresh (every game, after all type reads) | Generic fallback | Result seen in play |
|---|---|---|---|---|
| infantry, vehicle, aircraft (sprite) | theater flag: I ‖ .E. New-theater flag: subst(I ‖ .shp). Else I ‖ .shp | nothing | yes | last type read |
| infantry, vehicle, aircraft (voxel) | voxel parts (claim 16), no theater change | nothing | no | last type read |
| building body sprite | subst((art image key or I) ‖ .E or .shp), whatever the new-theater flag | theater flag: I ‖ .E. New-theater flag: as at the type read. Else nothing | yes at the type read, no for the theater-flag refresh | refresh where it acts, else last type read |
| building other sprites | subst(name ‖ .shp) | theater flag: the build-up sprite becomes build-up name ‖ .E, the others are not touched. New-theater flag: as at the type read | yes, except the theater-flag refresh | refresh where it acts, else last type read |
| overlay | as the infantry sprite | theater flag: I ‖ .E. New-theater flag: subst(I ‖ .shp) | yes | refresh (same names) |
| animation | theater flag: I ‖ .E. Else I ‖ .shp, no subst | theater flag: type name ‖ .E, then type name ‖ .shp. New-theater flag: subst(I ‖ .shp) | yes, except the theater-flag refresh | refresh where it acts, else last type read |
| terrain object | theater flag: nothing. Else I ‖ .shp | theater flag: type name ‖ .E | no | refresh for the theater flag, else type read |
| smudge | theater flag: I ‖ .E. Else I ‖ .shp | theater flag: type name ‖ .E | no | refresh for the theater flag, else type read |
| projectile | with the voxel flag: voxel parts (claim 27). Else as the infantry sprite, from the image name of claim 26 | nothing | yes for the sprite, no for the model | last type read |
| particle, particle system | as the infantry sprite; nothing with the voxel flag | nothing | yes | last type read |
| voxel animation | image ‖ .vxl, or the shared part (claim 30) | nothing | no | last type read |
| superweapon icon | sidebar icon name ‖ .shp, else the placeholder | nothing | no | last type read |

**Table 3 — state kept per type within one game** (all of it starts afresh with every game).

| State | Event | Result |
|---|---|---|
| non-projectile image name | type read with a non-empty image key | the key's value, cut to 24 bytes, then any eligible arctic suffix (claims 9, 10) |
| non-projectile image name | type read without a non-empty image key | kept, then any eligible arctic suffix (claims 9, 10) |
| arctic flag set after the previous read (snow): no (also before the first read) | type read after which the flag is set | A appended (claims 9, 10); now "yes" |
| arctic flag set after the previous read (snow): yes | type read after which the flag is set | nothing appended, whatever the image name now is; stays "yes" |
| arctic flag set after the previous read (snow): either | type read after which the flag is not set | nothing appended or removed; now "no" |
| main icon: not set | first type read | set from the art section, or the placeholder (claim 22) |
| main icon: set | any later type read | unchanged |
| alternate icon: none | type read whose art section's alternate icon file resolves | set (claim 25) |
| alternate icon: none | other type read | stays none |
| alternate icon: set | any later type read | unchanged |
| animation: previous type read found its sprite file: no (also before the first read) / yes | type read | flags per claim 2. "Yes" afterwards exactly when this read found the sprite file |
| building secondary sprite (each of eight): any | type read whose art section does not set its name, or sets it empty | unchanged (claim 11) |
| building secondary sprite: any | type read whose art section sets its name | the file found, or none (claim 11) |
| projectile image name: any | any type read | the read's image key, or empty without one (claim 26) |
| projectile trailer settings: any | type read with no non-empty image value | unchanged, even when its own picture is lost (claim 33) |
| voxel animation share flags and source: any | any type read | the read's values, or "not set" and "none" without them (claim 30) |
| superweapon icon: any | any type read | rebuilt from the sidebar icon name then in force (claim 31) |

The arctic rows apply only where claims 9 and 27 make the type eligible. For an animation,
only type reads after which its theater flag is set take part in those rows. Its other type reads append nothing and leave the arctic state unchanged (claims 9, 10).

Observable consequences of the order "theater set, type reads (base rules, language rules file,
game-mode layer, map layer, as far as the game has them), then theater refresh":
- In every game, including the first of a session, every name that claims 6 to 15 make depend on
  the theater uses that game's theater. No art named for an earlier game's theater remains.
- A map override that changes an overlay's or building's image name takes effect in the
  theater-dependent names, because the refresh runs after it.
- A theater-flagged building, animation, terrain object or smudge shows the file that the refresh
  gives it (for a building, its body and build-up sprites; its other sprites keep the type read's
  files). The type read's file for those sprites never reaches the screen.

## Edge cases and quirks

Each switch defaults to the original (YR) behaviour.

- **`quirks.anim_flags_follow_held_sprite`** (claim 2). YR: an animation takes its theater and
  new-theater flags from its own art section first only at the first type read of a game or at a
  type read whose previous type read did not find the sprite file. So when its image name differs from its type name and the two sections set a flag to
  different values, the flag in play depends on whether the second-to-last type read found the
  sprite file, and can depend on the number of type reads in the game (A9 part iii). Alternative:
  every type read takes each flag as the first row of claim 2's table gives it (the own art
  section's value if that section sets the flag; else the art section's value if that sets it; else
  the value from before the read), whatever the previous type read found.
- **`quirks.art_generic_fallback_any_name`** (claim 8). YR: a missing sprite is retried with G as
  second character, even when no theater rule changed the name. So a missing "FOO.shp" is retried as
  "FGO.shp", a missing "FOO.tem" as "FGO.tem", and "Q.shp" as "QGshp". Alternative: retry only names
  that letter substitution changed.
- **`quirks.arctic_suffix_on_flag_turning_on`** (claim 10). YR: on snow, the A is appended at a type
  read after which the alternate-arctic-art flag is set, only when the flag was not set after the
  type's previous read (or the read is the first). So a later image key loses the A for the rest of
  the game while the flag stays set, and a flag set, cleared and set again appends it twice. Alternative: the final image name, after all type reads,
  gets exactly one A if the flag is finally set and the theater is snow.
- **`quirks.building_substitution_ignores_flag`** (claim 11). YR: buildings always get letter
  substitution. Alternative: only with the new-theater flag.
- **`quirks.theater_art_uses_type_name`** (claims 13, 14). YR: terrain objects, smudges and
  animations with the theater flag build their refreshed file from the type name. Alternative: use the
  image name. The two settings give the same names whenever the image name equals the type name.
- **`quirks.theater_refresh_building_no_fallback`** (claim 12). YR: the theater-flag refresh of a
  building has no generic fallback and ignores the art section's image key, and a miss leaves the
  building without a body sprite. Alternative: keep the type read's result.
- **`quirks.apc_water_body`** (claim 16). YR: only a vehicle whose type name is exactly "APC" looks for
  the I ‖ "W" alternate body. Alternative: every vehicle with the voxel flag and without the turret
  flag looks for it, whatever its type name.
- **`quirks.voxel_part_failure_discards_all`** (claim 18). YR: a missing numbered turret, or any
  found but invalid part (or its motion file), discards the whole voxel art of the type, and nothing
  of it is drawn. Alternative: only that part is absent; only a missing or invalid body removes the
  type's voxel art.
- **`quirks.building_turret_without_tur_in_barrel_role`** (claim 20). YR: a turret-animation name
  without a late "TUR" (and no barrel-animation-is-voxel flag) becomes the barrel model. Alternative:
  load it as the turret model.
- **`quirks.building_tur_search_skips_four`** (claim 20). YR: "TUR" counts only from the fifth
  character of the turret-animation name. Alternative: an occurrence anywhere in the name counts.
- **`quirks.main_icon_first_read_only`** (claim 22). YR: the main icon is set at the first type read
  of a game, and later type reads never replace it, even when they change the image name, and with
  it the art section. Alternative: a later type read builds the main icon again by claim 22's rule
  from the art section of that read, and keeps the earlier icon where that art section does not set
  the icon name.
- **`quirks.alt_icon_kept_once_loaded`** (claim 25). YR: a type read loads an alternate icon only
  while the type has none, so once one is loaded, later type reads never replace it. Alternative: a
  later type read whose art section sets a non-empty alternate icon name whose file resolves loads
  that file in place of the earlier one; any other type read keeps the alternate icon the type has.
- **New-theater animations: the type-read sprite is never seen** (claim 14). In play such an
  animation shows the refresh's substituted sprite. The unsubstituted image ‖ .shp (or its generic
  fallback) that its type reads name never reaches the screen, so this is not a quirk and needs no
  switch. That name still matters through whether it resolves, because that decides the flags of
  the animation's later reads (claim 2). An implementation therefore need not load that picture,
  but must determine whether its name resolves. Backing and confidence are those of claims 2
  and 14; this entry introduces no separate rule.
- **Unused art-section image keys** (claim 3). For infantry, vehicles and aircraft an importer must
  ignore an image key inside the art section. Using it could select a different file from the
  rules-selected image name.
  Backing and confidence follow claim 3: A4 isolates infantry only.
- **Turret count above 18** (claim 17) and **invalid numbered barrel** (claim 18): the original's
  result is unspecified. Our engine clamps the count to 18 with an importer warning, and discards the
  voxel art for an invalid numbered barrel. These are departures, not quirk switches.
- **`quirks.projectile_image_afresh`** (claim 26). YR: a projectile's image name is its type read's
  image key or empty, so a later type read without the key leaves the projectile without art.
  Alternative: a later type read without an image key keeps the image name of the earlier read (the
  first read still gives an empty name without a key). With either setting the importer warns about an override section that drops the
  key.
- **`quirks.voxel_anim_share_afresh`** (claim 30). YR: the share flags and the share source of a
  voxel animation are read afresh at every type read, so an override section that omits them turns
  the animation to its own model file (with our engine's choice for the unspecified part of that
  case, claim 30), and one that names another source takes that source's part. Alternative: a later
  type read without these keys keeps the earlier values.
- **Projectile with an empty image name** (claim 26), **voxel animation whose share source is read
  again later**, **voxel animation that drops its share flags**, and **voxel animation whose own
  model is missing or invalid** (claim 30): departures, with importer warnings as stated in those
  claims. They are not quirk switches.

## Import mapping

**Later type reads.** The "Default when absent" column gives the value at the first type read of a
game. Unless a row says otherwise, a later type read that does not set a key keeps the value the
earlier read left (the glossary's *rules layer*). Rows marked **afresh** are read again at every
type read with the stated default; rows marked **frozen** follow the rule of the cited claim.

Rules database, the type's own section (section = type name). For animations the first three rows
come from the own art section instead (below the art-database table). For projectiles the `Image`
row is replaced by the projectile row below:

| Key | Parse | Feeds | Default when absent |
|---|---|---|---|
| `Image` | text, cut to 24 bytes | image name (claim 1) | the type name |
| `AlphaImage` | text, cut to 24 bytes | translucent sprite (claim 23) | empty |
| `AlternateArcticArt` | boolean | arctic alternate (claims 9, 10) | no |
| `NoSpawnAlt` | boolean | "WO" alternate body (claim 16) | no |
| `Turret` | boolean | turret parts (claims 16, 19) | no |
| `TurretCount` | integer | multiple-turret count n (claims 16, 17) | 0 |
| `IsGattling` | boolean | disables the numbered turrets (claim 16) | no |
| `TurretAnim` | text | building turret-animation name T (claim 20) | empty |
| `TurretAnimIsVoxel` | boolean | building voxel turret (claim 20) | no |
| `BarrelAnimIsVoxel` | boolean | building voxel barrel (claim 20) | no |
| `VoxelBarrelFile` | text | building voxel barrel name (claim 20) | empty |
| `Trainable` | boolean | trainable flag, alternate icon choice (claim 25) | yes |
| `UndeploysInto` | type name (a vehicle type) | a building's undeploy-into vehicle, alternate icon choice (claim 25) | none |

Art database, the art section (section = image name):

| Key | Parse | Feeds | Default when absent |
|---|---|---|---|
| `Voxel` | boolean | voxel or sprite (claim 4) | no |
| `Theater` | boolean | theater flag (claims 2, 6, 11 to 15) | no |
| `NewTheater` | boolean | new-theater flag (claims 2, 7, 11, 12, 14, 15) | no |
| `Image` | text, cut to 63 bytes | building body sprite name only (claims 3, 11); the animation's own art section is covered below the table | empty, **afresh**: the image name is then used |
| `Buildup` | text, cut to 15 bytes | building build-up sprite (claims 11, 12) | empty (no sprite); an empty value also keeps the earlier name |
| `DeployingAnim`, `RoofDeployingAnim`, `DoorAnim`, `UnderDoorAnim`, `UnderRoofDoorAnim`, `Rubble`, `SpecialZOverlay`, `BibShape` | text, cut to 63 bytes | building secondary sprites (claim 11) | empty (no sprite); at a later read, absent or empty keeps the sprite already held (claim 11) |
| `Cameo` | text, cut to 24 bytes | main icon (claim 22) | empty → placeholder; **frozen** (claim 22) |
| `AltCameo` | text, cut to 24 bytes | alternate icon (claim 25) | empty → none; **frozen** once loaded (claim 25) |

Animations (defined in the art database). Every animation property is read from the art database,
the same in every rules layer (S-0006 claim 8). The own art section (section = animation type name)
gives, with the parse forms, defaults and later-read rules of the rules-database rows above:

| Key | Parse | Feeds | Default when absent |
|---|---|---|---|
| `Image` | text, cut to 24 bytes | image name (claim 1) | the type name |
| `AlphaImage` | text, cut to 24 bytes | translucent sprite (claim 23) | empty |
| `AlternateArcticArt` | boolean | arctic alternate (claims 9, 10) | no |
| `Theater`, `NewTheater` | boolean | theater and new-theater flags, by claim 2's table | no |

The art section (section = image name) gives `Theater` and `NewTheater` as in the art-database
table, by claim 2's table.

Projectiles, voxel animations and superweapons, rules database, the type's own section:

| Key | Parse | Feeds | Default when absent |
|---|---|---|---|
| `Image` (projectile) | text, cut to 24 bytes | projectile image name (claim 26); unsuffixed trailer section (claim 33) | empty, **afresh** (no body art; trailer settings not read) |
| `Inviso` (projectile) | boolean | invisible projectile (claim 27) | no |
| `ShareBodyData`, `ShareTurretData`, `ShareBarrelData` (voxel animation) | boolean | share flags (claim 30) | no, **afresh** |
| `ShareSource` (voxel animation) | type name, matched against the existing infantry, vehicle, aircraft and building types; naming a type creates none, and a name that matches none gives no share source | share source (claim 30) | none, **afresh** |
| `SidebarImage` (superweapon) | text, cut to 24 bytes | superweapon sidebar icon name (claim 31) | the type name; rebuilt at every type read |

Particles, particle systems and voxel animations use the general rows (`Image`, `AlphaImage`,
`AlternateArcticArt`; the art section's `Theater`, `NewTheater`, `Voxel`). Weapons, warheads and
tiberiums feed no art file (claim 32); a tiberium's `Image` is a number and belongs to a tiberium
spec.

Art-file references and trailer settings (claim 33). These rows require the current rules layer
to contain the referring type's rules section. The selected section uses the current rules `Image`
for buildings and aircraft, including the suffix when claims 9 and 10 add it. For projectiles it
uses the current non-empty rules `Image` before drawing-name suffixing; absent or empty `Image`
skips both trailer rows and keeps their earlier values. An art-section `Image` that names a
building's body sprite does not select these reference sections.

| User data | Parse | Feeds | Default when absent |
|---|---|---|---|
| Art, selected building image section, `ToOverlay` | overlay name, creating, cut to 127 bytes (S-0004) | convert-to-overlay reference; S-0004 claim 14 | none; later absent or empty keeps held reference |
| Art, selected aircraft or projectile image section, `Trailer` | animation name, creating, cut to 127 bytes (S-0004) | trailer animation; S-0004 claim 14 | none; later absent or empty keeps held reference; reserved no-type value clears |
| Art, selected aircraft or projectile image section, `SpawnDelay` | integer ticks (S-0003) | trailer delay; projectile timing is S-0306 claim 16 | 3 ticks; later missing or empty keeps held delay |

The map's theater (S-0009) selects the theater row of Table 1.

The tables' parse forms follow S-0003. Backing and confidence for the mapped behaviour follow
its cited claim. Absent-key defaults not isolated by those scenarios are `unmeasured`, confidence
high: explicit settings in the fixtures do not expose what omission would do, and no separate
consumer scenario is proposed for each such default here.

## Measurement scenarios

**Common setup.** Every scenario uses the glossary's measurement setup: an unmodified install with no
loose override files and no engine add-ons. Scenarios marked **test data** add files or sections, and
say where:
- in the map: a tiny purpose-made test map, which may carry override sections (S-0006).
- in the test expansion archive: test files, or a copy of the art database with the stated change
  (S-0005 says this copy wins over the stock one).
- to make a stock file *absent*, which no higher-ranked copy can do: a loose copy of the stock nested
  archive that holds it, minus that one file (S-0005 claim 3).
- The test expansion archive ranks below the two language archives (S-0005 claim 9), so it cannot
  replace a member at the top level of either of them, such as the expansion's nested icon archive
  or its string table. A scenario that needs to replace or remove such a member, or a file inside
  the stock icon archive, uses a loose file, or a loose copy of the nested archive (S-0005 claims 1
  and 3). No scenario below does: every icon file they add has a new name. The art database and the
  game-mode files rank below the expansion archives, so their copies in the test expansion archive
  win.

All test data is removed afterwards. Use a named loose copy of the base rules and an art database copy in the test expansion
archive for each fixture, plus the stated game-mode copy. A **test type** is a
copy of a usable type definition, with every art-related field stated by the scenario explicitly
set or omitted in these copies. The fixture, not the shipped definition, supplies these properties.
Use invented image names absent from the install, supply all stated valid files as copies of
compatible owned assets, and supply distinguishable pictures wherever alternatives are compared.
Names used as missing files must also have missing generic candidates. Unless varied, clear the
theater, new-theater, arctic and voxel flags, use sprite art, and give explicit image names.
List test types in the base rules so they receive the stated reads, except where a scenario
explicitly adds a type in the map. For weapons and projectiles, establish the references in the
base rules instead. Later sections contain an unrelated valid entry as well, so omitting or
emptying a tested key does not remove the section itself (S-0001).

Every run is a skirmish at game speed 2 and 800×600, director off and mods off except the named
test data. Invented names use ASCII bytes, so the scenarios' character counts equal byte counts.
One human player owns the observed objects; one bystander computer player stays
outside the observed map patch or firing corridor, with the glossary's void rule. All observed
objects are in view unless fog is explicitly tested. Start a fresh session for each variant to
avoid the per-session art store, and repeat A1 and A8 in a second game of the same session to
test game-start refresh. Art lookup is deterministic: one run per map and variant, apart from
A22's debris repetitions. Capture screenshots, the sidebar or tick-labelled video as appropriate.
A positive control that explicitly selects each candidate picture must show it. Otherwise a
missing picture is inconclusive, not evidence for a lookup rule. Any result other than a
scenario's stated expected picture, absence or timing falsifies its tested rule, once controls
pass. The scenarios describe imposed test inputs, not properties of shipped types or files.

**Pictures compared across theaters.** Each theater has its own unit, tile and theater palettes
(glossary, *palette*), so one sprite file can look different on two theaters. Where a scenario
compares an object's pictures on maps of different theaters, and the palette that object is drawn
with differs between those theaters (S-0103), "the same picture" means the same image frames
whatever their colours, and "different pictures" means a difference that the change of palette
alone cannot produce. A test object of the same kind, drawn with the same palette, whose file name
does not depend on the theater, placed on the same maps, shows what the palette alone changes. For
example, in A1 a building of part (b) drawn with the same palette as the building of part (a) is
that control, and in A6 part (a) a test infantry type without the alternate-arctic-art flag, the
theater flag or the new-theater flag is.

- **A1 — building letter substitution without the new-theater flag (claims 5, 7, 11, 24).**
  - Setup: test data. Four tiny maps, temperate, snow, urban and desert, with the same test building.
    Set its image name to an invented name beginning GA, clear both theater flags, and provide
    distinct body pictures under the GT, GA, GU and GD variants of that name.
  - Procedure: start each map, also as the first game of a fresh session, and screenshot the
    building.
  - Observable: the building's picture on each map.
  - Expected: four different, theater-specific pictures.
  - Falsified if all four show the same picture.
  - (b) Negative case of claim 7. Setup: the same maps with a test building whose theater flag is
    cleared and body name begins ZZ, with one supplied body sprite. Observable: its main picture
    on each map. Expected: the same main
    picture on all four. Falsified if any two differ.
- **A2 — theater extension and refresh (claims 6, 11, 12, 13, 15).**
  - Setup: test data. The same maps with test terrain, overlay and smudge types whose theater
    flag is explicitly set, and a test building whose new-theater flag is set, theater flag cleared
    and build-up name supplied. Provide distinguishable files for every expected theater name.
    Repeat the overlay test with only the new-theater flag set and distinct letter-substituted files.
  - Procedure: start each map, screenshot all four objects, and watch the building's build-up when a
    copy of it is placed.
  - Observable: each object's picture and the build-up animation on each map.
  - Expected: the supplied picture for each object's theater-specific name.
  - Falsified if the tree, overlay or smudge is missing on a theater that has its file (type name ‖
    .E for the tree and the smudge, image ‖ .E for the overlay), or if the building or its build-up looks the same on all four theaters.
  - (b) No letter substitution on theater-extension names (claim 6). Test data, temperate map: in
    the art database copy, a section "GAZZO" for a test overlay with the
    theater flag explicitly set; in the map, an override section setting that overlay's image key to "GAZZO". In
    the test expansion archive, "GAZZO.tem" (picture P) and "GTZZO.tem" (picture Q, a clearly
    different overlay picture). Observable: the overlay's picture. Expected: P. Falsified if Q shows.
- **A3 — generic fallback (claims 7, 8).**
  - (a) Setup: test building with an invented GA image name, both theater flags cleared. Supply
    only its GA and GG body sprites, with distinct pictures. Run temperate and snow maps.
    Expected: GG on temperate, GA on snow. Any other result falsifies. Repeat for each eligible
    first letter G, N, C and Y, each eligible second letter A and T, and lower-case names.
  - (b) Test data: in the art database copy, a section "FOO" copied from a test infantry's art
    section. In the test expansion archive, a sprite "FGO.shp" (a copy of a different infantry's
    sprite) and no "FOO.shp". In the map, an override section setting that infantry's image key to
    "FOO". Expected: the infantry shows the FGO picture. Falsified if it is invisible.
  - (c) Test data: as (b), with the section and image key "Q" and a sprite file named "QGshp" (no
    dot). Expected: the infantry shows the QGshp picture. Falsified if it is invisible.
  - (d) Theater extension. Test data: in the art database copy, a section "FOO" copied from the art
    section of a test overlay with the theater flag explicitly set. In the map, an override section setting that
    overlay's image key to "FOO", on a temperate map where the overlay is placed. In the test
    expansion archive, "FGO.tem" (a copy of a clearly different overlay picture) and no "FOO.tem".
    Expected: the overlay shows the FGO picture. Falsified if it is invisible.
  - (e) As (b), with both FOO.shp and FGO.shp absent. Expected: invisible. A visible body
    falsifies the final missing-file outcome, provided a supplied-file control shows the body.
  - Observable: the object's picture. Runs: one per map and variant.
- **A4 — image indirection, unused art-section image keys, letter case, length cut (claims 1, 3, 5,
  24).**
  - (a) Test data: in the art database copy, give a test infantry's art section an image key that
    names another infantry's sprite. Expected: the infantry looks unchanged. Falsified if it looks
    like the other infantry.
  - (b) Test data: in the map, an override section giving the same infantry an image key that names
    the other infantry. Expected: it looks like the other infantry. Repeat with the value written in
    lower case: same result. Falsified if it looks unchanged, or if the two cases differ.
  - (c) Test data: let N be an invented name of exactly 24 characters. In the art database copy, a
    section N copied from the infantry's art section. In the test expansion archive, a sprite N ‖
    .shp copied from the other infantry's sprite. In the map, an override section setting the
    infantry's image key to N followed by six more characters. Expected: the infantry shows the N
    picture. Falsified if it is invisible.
  - (d) In the test base rules, omit the infantry's image key and provide type name ‖ .shp.
    Expected: that picture. Then give the base read an explicit different image and let the map
    section omit the image key, then set it empty in a separate run. Expected: the explicit
    picture remains in both. A contrary picture or absence falsifies each case.
  - Observable: the infantry's picture. Runs: one per variant.
- **A5 — voxel parts (claims 4, 16, 18).** Two kinds of change are used: an *invalid* part is an
  empty file under the part's name in the test expansion archive; a *removed* part is omitted
  from the test archive. Invented image names ensure there is no lower-ranked copy. Observable: whether the vehicle is drawn, and which parts it
  shows. Runs: one per variant.
  - (a) A test tank with voxel and turret flags set, multiple-turret count 0, and valid body,
    turret and barrel model/motion pairs supplied: invalid turret motion file (I ‖ "TUR" ‖ .hva). Expected: the tank
    is not drawn at all. Falsified if it is drawn.
  - (b) The same tank: invalid barrel motion file (I ‖ "BARL" ‖ .hva). Expected: not drawn.
    Falsified if drawn.
  - (c) The same tank: barrel model removed. Expected: drawn, without a barrel. Falsified if not
    drawn.
  - (d) A test vehicle with voxel and turret flags set, multiple-turret count 2, gatling flag
    cleared, and all named pairs supplied: turret 1
    model (I ‖ "TUR1") removed. Expected: not drawn. Falsified if drawn.
  - (e) The same vehicle: supplied barrel 1 model (I ‖ "BARL1") removed. Expected:
    drawn. Falsified if not drawn.
  - (f) A test voxel vehicle with the turret flag cleared, no-spawn-alternate flag set and valid
    body and I ‖ "WO" pairs supplied: invalid I ‖ "WO" ‖ .hva. Expected: not drawn. Then instead remove its I
    ‖ "WO" model. Expected: drawn. Falsified by either other outcome.
  - (g) Test data: the map lists a new vehicle type "APC" with a full definition copied from a stock
    vehicle, with its image key omitted and the turret and no-spawn-alternate flags explicitly
    cleared, and a second new type "APD" defined the same way (S-0006 claim
    7). The art database copy has sections "APC" and "APD" with the voxel flag set. The test
    expansion archive has valid body files APC and APD (.vxl and .hva copies of a stock vehicle), and
    APCW.vxl and APDW.vxl with *empty* motion files APCW.hva and APDW.hva. Place one of each.
    Expected: "APC" is not drawn and "APD" is drawn. Falsified if both are drawn or neither is.
  - (h) Test data: the test tank of (a) with its body model removed, and a body sprite I ‖ .shp
    (copy of a stock vehicle sprite) added. Expected: not drawn. Falsified if the sprite shows.
  - (i) The tank of (a): its turret motion file (I ‖ "TUR" ‖ .hva) removed, the turret model kept.
    Expected: not drawn. Falsified if drawn.
- **A6 — arctic alternate (claim 9).** Setup: test data. Snow and temperate maps with a test
  infantry type buildable from the sidebar, arctic flag explicitly set, other art flags cleared.
  Supply distinct unsuffixed and A-suffixed sprites and art sections. Give the base rules this
  type's only rules section, except where a variant requires more reads.
  - (a) Observable: the infantry's picture. Expected: different pictures on snow and temperate.
    Falsified if they are the same.
  - (b) Test data: in the art database copy, give the suffixed art section (image ‖ "A") an icon name
    that names another unit's sidebar icon. Observable: the sidebar icon. Expected: the other icon on
    snow, the usual icon on temperate. Falsified if the icon is the same on both.
  - (c) Test data: in the art database copy, set the voxel flag in the suffixed art section only.
    The skirmish is started in a mode whose game-mode file has no section for the infantry, and the
    map has no override section for it, so the base rules give its only type read (a later type read would take the voxel flag from the
    suffixed art section, claims 2 and 9). Expected: on snow the infantry is still drawn as its
    arctic sprite. Falsified if it is invisible
    on snow.
  - Runs: one per map and variant.
- **A7 — arctic suffix across type reads (claims 9, 10).** Setup: snow test maps with the same
  infantry as A6, and test data as stated. Every infantry type used here is named by the base rules'
  infantry list, so it has a type read in the base rules (Terms, *type read*). Observable: the infantry's picture. Runs: one per map.
  - (a) The map has an override section for the infantry that sets its image key to its unsuffixed
    image name (the explicit value in the test base rules). A second
    map has none. Expected: the arctic picture without the override, the
    non-arctic picture with it. Falsified if both show the arctic picture.
  - (b) The override section sets another key unrelated to art, and no image key. Expected: the
    arctic picture. Falsified if it shows the non-arctic picture.
  - (c) For a test infantry whose base-rules arctic flag is explicitly cleared: the override section sets the flag, and the
    test expansion archive and art database copy provide image ‖ "A" (sprite and art section copied
    from another infantry). Expected: the other infantry's picture on snow. Falsified if the
    infantry looks unchanged.
  - (d) The skirmish is started in a mode that has a game-mode file. A test copy of that game-mode
    file (in the test expansion archive) has an override section that clears the flag for the infantry of A6, and the map's override section sets it again, neither with an image key. The test files
    image ‖ "AA" (sprite and art section copied from another infantry) are added. Expected: the
    other infantry's picture. Falsified if the usual arctic picture shows.
  - (e) Suffix state after an image-key read. The skirmish is started in a mode that has a game-mode
    file. A test copy of that game-mode file (in the test
    expansion archive) has an override section that sets the infantry's image key to its unsuffixed
    image name. The map has an override section for the infantry that sets an unrelated key and no
    image key. Neither touches the flag, and the infantry has three type reads (base rules,
    game-mode file, map). Expected: the non-arctic picture (the map read appends
    nothing, because the flag was already set after the game-mode read). Falsified if the arctic
    picture shows.
- **A8 — animation theater rules (claims 14, 24).**
  - Setup: test data. Tiny temperate, snow and desert maps. A test building continuously plays
    a test animation with only the new-theater flag set and an invented GA image name. Place a
    second test animation W with only the theater flag set. List both in the test base rules.
    Supply distinct pictures for the substituted animation names and W's type name ‖ .E files.
  - Procedure: start each map, including as the first game of a fresh session, and screenshot the
    building's animated part and W.
  - Observable: their pictures per theater.
  - Expected: theater-specific pictures in every theater, including the first game.
  - Falsified if the building's animation shows the same picture on all three theaters, or if W
    shows a picture other than the supplied W type name ‖ .E picture, or is invisible.
  - Optional test data (claim 14, refresh by type name with the .shp retry), temperate map only. Two
    conditions make it discriminate: the image name must differ from the type name, or the
    image-named and type-name-named files would be the same files; and the theater flag must still
    be set after the game's last type read (claim 2), or the refresh would not take the theater-flag
    path. So: in the art database copy, W's own art section gets an image key naming an invented
    name "ZZW", and the copy has **no** section "ZZW", so every read takes the theater flag from W's
    own art section (claim 2). In the test expansion archive: "ZZW.tem" (picture P) and W's type
    name ‖ .shp (picture S, a copy of a clearly different sprite). Remove the fixture's W type
    name ‖ .tem file from the test archive; W's invented type name has no lower-ranked copy. Expected: S. Falsified if P shows, if W's usual picture shows, or if W is invisible.
  - Runs: one per map.
- **A9 — animation flags from two sections (claims 2, 3).**
  - Setup: test data on a temperate map with a test building continuously playing animation X.
    Explicitly set X's own new-theater flag and clear its theater flag. In the art database copy: X's own art section gets the
    image key "GAZZT" (an invented name that letter substitution changes). A new section "GAZZT"
    sets the new-theater flag to *no* and has its own image key "ZZOTHER". In the test expansion
    archive: "GTZZT.shp" (picture Q) and "ZZOTHER.shp" (picture R), and, for variant (i) only,
    "GAZZT.shp" (picture P). No file named "GGZZT.shp".
  - (i) With GAZZT.shp. Expected: P, because the first type read finds the sprite and the later reads
    take the flag from the "GAZZT" section. Falsified if Q shows.
  - (ii) Without GAZZT.shp. Expected: Q, because no read finds a sprite, so the own art section's
    new-theater flag stays in force and the refresh loads the substituted name. Falsified if the
    animation is invisible.
  - In both variants R never shows (claim 3). Falsified if it does.
  - Observable: the picture of the building's animated part. Runs: one per variant.
  - (iii) Only the previous read counts. Separate test data on a temperate map
    with a test animation X placed on the map (as in A8), listed in the test base rules. Set
    X's own theater flag and supply X ‖ .tem with picture R′. Neither section below sets the
    new-theater flag.
    In the art database copy: X's own art section gets the image key "ZZQ", and a new section "ZZQ"
    sets the theater flag to *no*. In the test expansion archive: "ZZQ.tem" (picture P′, a clearly
    different sprite). No "ZZQ.shp" and no "ZGQ.shp" exist.
    - (iii-a) A skirmish. Every skirmish gives X three type reads (base rules, game-mode layer, map
      layer; Terms, *type read*), whether or not its mode has a game-mode file. Read 1 is the first, so it takes the own
      section's theater flag and finds ZZQ.tem. Read 2 follows a read that found the sprite file, so
      it takes the "ZZQ" section's cleared flag and finds nothing. Read 3 follows a read that found
      nothing, so the theater flag is set once more, and the refresh loads X ‖ .tem. Expected: R′. Falsified if the animation is invisible or shows P′.
    - (iii-b) The same skirmish with a named empty language rules file present before session
      start. This adds one applied layer (S-0006), so X has four type reads. The last read leaves
      the theater flag cleared and no sprite. Expected: invisible. Falsified if R′ or P′ shows.
    - The hypothesis predicts R′ after three reads and nothing after four. Thus (iii-a) and
      (iii-b) must differ, while a rule based on whether any earlier read ever found a sprite
      would leave both invisible.
    - Observable: the animation's picture. Runs: one per variant.
- **A10 — building voxel turret names (claim 20).** Test data. In the map, override sections give a
  test defence with the turret-animation-is-voxel flag set and barrel flag cleared the
  turret-animation names below. The test expansion archive has the named files as copies of two
  clearly different stock models, M and N, with their motion files. Observable: how many models the
  defence shows, and which. Runs: one per name.
  - (a) T = "ABCDTUR", files ABCDTUR (M) and ABCDBARL (N). Expected: M and N both show.
  - (b) T = "ABCDTURXY", files ABCDTURXY (M) and ABCDBARL (N). Expected: M and N both show.
    Falsified, in (a) or (b), if only one model shows.
  - (c) T = "ATURB", files ATURB (M) and ABARL (N). Expected: only M shows (as the barrel model).
    Falsified if N shows.
  - (d) As (c) with the barrel-animation-is-voxel flag set and the voxel barrel file name "ZZBAR",
    file ZZBAR (N). Expected: only N shows. Falsified if M shows.
- **A11 — terrain objects and smudges without the theater flag (claim 13).**
  - (a) Setup: test data. Temperate and snow maps with two test smudges, theater flag cleared,
    explicit image names and supplied sprites. Expected: drawn, with the same frames on both.
    Falsified if invisible.
  - (b) Test data: in the art database copy, clear the theater flag of a test terrain object. In the test
    expansion archive, image ‖ .shp (a copy of a different tree's temperate picture). Temperate and
    snow maps with the tree. Expected: the copied picture on both maps. Falsified if the tree is
    invisible or theater-specific.
  - (c) As (b) but the file is named with G as second character instead (generic name) and image ‖
    .shp is absent. Expected: the tree is invisible (no fallback). Falsified if it shows.
  - Observable: the object's picture. Runs: one per map and variant.
- **A12 — building art-section names and the theater-flag refresh (claims 11, 12).**
  - (a) Test data: in the art database copy, give a test building the theater flag and an art-section
    image key J naming an invented file name whose first two letters are not a pair of claim 7. In
    the test expansion archive, J ‖ .tem (a copy of the building's picture). No file image name ‖
    .tem exists.
  - Observable: whether the building's main picture is drawn on a temperate map.
  - Expected: no main picture, because the refresh looks only for image name ‖ .tem, although the
    type read found J ‖ .tem. Then add image name ‖ .tem (a clearly different picture): that picture
    shows.
  - Falsified if the J picture shows in the first run, or if the added image name ‖ .tem picture
    does not show in the second.
  - (b) Test data: in the art database copy, give a test building with the theater flag explicitly cleared an
    art-section image key J (as above) and add J ‖ .shp (a clearly different picture) to the test
    expansion archive. Expected: the building shows the J picture. Falsified if it shows its usual
    picture.
  - (c) Length cut of the art-section image key. As (b), but J is an invented name of exactly 63
    characters, the image key holds J followed by five more characters, and the file in the test
    expansion archive is J ‖ .shp. Expected: the J picture. Falsified if the building has no main
    picture.
  - (d) Length cut of the build-up name. Test data: in the art database copy, set the build-up name
    of a test building with the theater flag cleared to B followed by four more characters, where B is
    an invented name of exactly 15 characters whose first two letters are not a pair of claim 7. In
    the test expansion archive, B ‖ .shp (a copy of a clearly different building's build-up sprite).
    Procedure: place the building. Expected: the copied build-up plays. Falsified if no build-up or
    the usual one plays.
  - (e) Afresh image key, kept build-up name. Test data: a test building with the theater flag
    cleared and its initial art section explicitly giving an image key and build-up name.
    Supply the corresponding body and build-up sprites. In the art database copy, a section "ZZB" copied from
    that art section without the image key and without the build-up name. In the map, an override
    section sets the building's image key to "ZZB". In the test expansion archive, "ZZB.shp" (a
    clearly different building picture). Procedure: place the building. Expected: the building shows
    the ZZB picture, and its usual build-up plays. Falsified if it shows the picture named by the
    first image key, or if no build-up plays.
  - Runs: one per variant.
- **A13 — refresh by type name (claims 13, 14).** Test data: on temperate, override sections in the
  map set the image keys of test terrain and smudge types, whose theater flags are set, to
  invented names different from their type names. Supply distinct pictures for the type-named
  and image-named theater files. Animations ignore map sections (S-0006
  claim 8), so for them the art database copy gives a test animation's own section a set theater
  flag and an image key naming a different invented art section with that flag also set.
  Supply distinct type-named and image-named theater files. All three types are named by
  the base rules' type lists. Observable: their pictures.
  Expected: each shows its own usual picture. Falsified if it shows the other object's picture.
  Runs: one.
- **A14 — sprite body with a voxel turret (claim 19).**
  - Test data: in the art database copy, clear the voxel flag of a test tank with its turret flag explicitly set, and add
    its body sprite I ‖ .shp (a copy of a stock vehicle sprite) to the test expansion archive.
    Supply valid I ‖ "TUR" and I ‖ "BARL" model/motion pairs for the control.
  - (a) Expected: the body sprite is drawn. Falsified if the vehicle is invisible.
  - (b) Add an empty I ‖ "TUR" ‖ .hva. Expected: the body sprite is still drawn. Falsified if the
    vehicle is invisible.
  - Observable: whether the body sprite is drawn, and whether a voxel turret appears on it (which
    also informs the rendering specs). Runs: one per variant.
- **A15 — no separate shadow file (claim 21).** Test data: in the test expansion archive, a copy of a
  stock infantry's sprite in which every image frame of the second half is made fully transparent.
  Observable: the infantry and its shadow on a plain map. Expected: the infantry is drawn normally
  and has no shadow. Falsified if a shadow is still drawn. Runs: one.
- **A16 — sidebar icon (claim 22).**
  - (a) Test data: in the art database copy, point a buildable unit's icon name at a non-existent
    file. Expected: the placeholder icon on the sidebar. Falsified if the unit's usual icon or no
    icon shows.
  - (b) Test data: provide two test art sections with distinct body and icon sprites. In the map,
    an override section sets the unit's image key to the second section's name. Expected: the unit in play looks like the other
    unit, but its sidebar icon stays its own. Falsified if the icon changes.
  - (c) Length cut. Test data: in the art database copy, set the unit's icon name to C followed by
    four more characters, where C is an invented name of exactly 24 characters. In the test
    expansion archive, C ‖ .shp (a copy of another unit's icon). Expected: the other unit's icon.
    Falsified if the placeholder shows.
  - Observable: the sidebar icon and the unit's picture. Runs: one per variant.
- **A17 — translucent sprite (claim 23).**
  - Test data: give a test building an invented translucent-sprite name in the map section.
    Supply that named file as a copy of a compatible light-glow sprite.
  - (a) On temperate and snow maps. Expected: the same translucent picture on both. Falsified if it
    is missing or differs.
  - (b) The key names an invented file "GAZZL" while the test expansion archive holds only
    "GGZZL.shp" and "GTZZL.shp". Expected: no translucent sprite. Falsified if one shows.
  - (c) The key names an invented name of 24 characters followed by four more, and the test
    expansion archive holds a file named by the first 24 ‖ .shp. Expected: that picture shows.
    Falsified if nothing shows.
  - (d) Animation. Test data: configure a test building to play a test looping animation
    continuously. In the art database copy, give that animation's own section an invented
    translucent-sprite name, supplied as a compatible light-glow sprite copy; in a second
    run, the same name is set in a map override section for that animation instead. Use a night
    lighting setting and fog of war on, with the animation's place fogged, so that the glow is
    easy to see (S-0110 claim 6).
    Expected: in the first run, the glow of that translucent sprite appears around the animation
    (the objects and fog under it are brightened as by S-0104 claim 15 and S-0110 claim 6); in the
    second run, no glow (the rules layer cannot reach an animation's art section). Falsified if the
    first run shows no glow or the second run shows one.
  - Observable: the translucent sprite at the building; in part d, the glow at the animation.
    Runs: one per map and variant.
- **A18 — alternate icon (claim 25).**
  - Setup: test data. A small map with the player's infiltrating unit and an enemy barracks and
    war factory within reach. The player has their own barracks, war factory and aircraft factory.
    Provide buildable test infantry, vehicle and aircraft types with trainable flags explicitly
    set. Give their art sections distinct main and alternate icon names and supply each named
    sprite. Use only the named enemy buildings for infiltration; keep other enemy activity away.
  - Procedure: note the three sidebar icons. Send an infiltrating unit into the enemy barracks and
    note them again, then into the enemy war factory and note them again.
  - Observable: the three sidebar icons at each step.
  - (a) Expected: the infantry icon changes to its alternate icon after the barracks infiltration,
    the vehicle icon only after the war factory infiltration, and the aircraft icon never. Falsified
    by any other pattern.
  - (b) Test data: the map's override section clears the trainable flag of the infantry type.
    Expected: its icon never changes. Falsified if it changes.
  - (c) Test data: in the art database copy, the vehicle's alternate icon name names a non-existent
    file. Expected: after both infiltrations the vehicle still shows its main icon. Falsified if the
    placeholder or no icon shows.
  - (d) Late setting. Test data: as (c), and also a section "ZZV" in the art database copy, copied
    from the vehicle's art section, whose alternate icon name names another unit's icon file, plus
    the vehicle's art files (each model and motion file, or its sprite) copied with ZZV in place of
    its image name into the test expansion archive. The map's
    override section sets the vehicle's image key to "ZZV". Expected: before the infiltrations the
    vehicle's main icon is its usual one (claim 22); after the war factory infiltration it shows the
    other unit's icon. Falsified if it still shows the main icon after the infiltration.
  - (e) A test building type with trainable flag set, undeploy-into vehicle explicitly named and
    supplied main and alternate icons. Include a control building without an undeploy reference:
    its main icon must remain after either infiltration. Expected: its icon changes after the war
    factory infiltration and not after the barracks infiltration. Falsified otherwise.
  - Runs: one per variant.
- **A19 — building secondary sprites at a later type read (claim 11).**
  - Setup: test data on a temperate map. Provide test building B with the theater flag cleared,
    its art section explicitly giving a bib name and omitting its own image key. Supply that bib. In the art database copy, a new section
    "ZZB" copied from B's art section without the bib name. In the test expansion archive, "ZZB.shp"
    (a copy of B's body sprite, so that the body looks the same). In the map, an override section
    sets B's image key to "ZZB". B is placed on the map.
  - (a) As set up. Expected: B's usual bib is drawn. Falsified if B has no bib.
  - (b) "ZZB" sets the bib name to an empty value. Expected: the usual bib is drawn. Falsified if
    there is no bib.
  - (c) "ZZB" sets the bib name to the invented name "ZZNOBIB", and no file "ZZNOBIB.shp" or
    "ZGNOBIB.shp" exists. Expected: no bib. Falsified if the usual bib is drawn.
  - (d) "ZZB" sets the bib name to a second invented name with a supplied distinct bib. Expected:
    the second bib. Falsified if the usual bib or no bib is drawn.
  - Observable: the bib under B. Runs: one per variant.
- **A20 — projectiles (claims 26, 27).**
  - Setup: a temperate map with a unit of the player that fires a slow, clearly visible sprite
    projectile (for example an artillery shell or a missile drawn as a sprite) whose rules section
    sets an image key, and a target. Variants (c) to (e) use a weapon with a voxel projectile or a
    sprite projectile as stated. Screen captures of the projectile in flight.
  - (a) The map's override section for the projectile sets an unrelated key and no image key.
    Expected: the projectile is not drawn in flight (its hit effects still show). A control run
    without the override shows it. Falsified if it is drawn.
  - (b) The override section sets the projectile's image key to a second test art section,
    with a distinguishable supplied sprite. Expected: the other projectile's picture. Falsified if the usual picture
    shows.
  - (c) A test projectile with the voxel flag explicitly set and valid model/motion pairs: in the test expansion archive, an empty file under
    its image name ‖ .hva. Expected: the projectile is not drawn. Falsified if it is drawn.
  - (d) Snow map, the voxel projectile of (c) without the change: the override section restates its
    image key I and sets the alternate-arctic-art flag. The test expansion archive has I ‖ "A" ‖ .vxl
    and .hva, copies of a clearly different stock model. Expected: the different model on snow, the
    usual model on a temperate control map. Falsified if the snow map shows the usual model.
  - (e) Snow map, the sprite projectile of (a): the override section restates its image key I and
    sets the alternate-arctic-art flag. The test expansion archive has I ‖ "A" ‖ .shp, a copy of a
    clearly different sprite. Expected: the usual picture. Falsified if the different picture
    shows.
  - (f) A test weapon with its projectile's invisible flag explicitly set: the override section gives that projectile the
    image key of the sprite projectile of (a). Expected: still nothing drawn in flight. Falsified if
    the sprite shows.
  - Observable: the projectile in flight. Runs: one per variant, several shots each.
- **A21 — particles (claim 28).**
  - Setup: test data on a temperate map with a test weapon effect configured to emit listed particle
    type P. Supply visible particle sprite art and trigger the effect in view. When a map section
    re-reads a particle or particle system, restate its behaviour and emitted-particle reference
    to avoid unrelated resets (S-0006).
  - (a) In the art database copy, a section "FOO" copied from P's art section (or empty if P's
    image name has none). In the map, an override section sets P's image key to "FOO". In the test
    expansion archive, "FGO.shp" (a copy of a clearly different particle sprite) and no "FOO.shp".
    Expected: the particles show the FGO picture. Falsified if they are invisible or look as usual.
  - (b) In the art database copy, P's art section (the section named by its image name, added if
    absent) sets the voxel flag. Expected: P's particles are not drawn. Falsified if they are drawn.
  - Observable: the particles. Runs: one per variant.
- **A22 — voxel animations (claim 30).**
  - Setup: test data on a temperate map. Configure a test vehicle's destruction to throw only
    listed voxel animation V, with no share flags in the base rules and valid body art supplied.
    Use an explicit debris count and trigger three destructions per variant. An unchanged-art
    control must show V on all three; otherwise the run cannot judge invisible debris.
  - (a) In the map, an override section sets V's image key to "ZZVA". In the test expansion archive,
    "ZZVA.vxl" (a copy of a clearly different stock voxel animation's model) and **no** "ZZVA.hva".
    Expected: the debris shows the copied model. Falsified if it is invisible or looks as usual.
  - (b) The override section for V sets the body share flag and names a test vehicle with voxel flag set and a valid supplied
    body pair as share source. Expected: the debris shows that vehicle's body. Falsified otherwise.
  - (c) A test voxel animation W configured in the base rules to share a test vehicle's supplied
    turret. Configure destruction to emit only W. The map section sets the turret share flag
    again and names another test vehicle with a distinguishable supplied voxel turret. Expected: W shows that other vehicle's turret.
    Falsified if it shows the usual turret.
  - (d) Flag priority. The override section for V sets the body and the turret share flags and
    names a test vehicle with voxel and turret flags set and supplied body/turret pairs as share source. Expected: the debris
    shows that vehicle's body. Falsified if it shows the turret. Then the section sets the turret and
    the barrel share flags instead, with a test vehicle having voxel and turret flags set and supplied turret/barrel pairs.
    Expected: the turret. Falsified if the barrel shows.
  - (e) Kind priority. Test data: the map's type lists add one new type name "ZZS" to two kinds at a
    time (S-0006 claim 7, as in A5 part g). Both types read the one rules section "ZZS", which does
    explicitly clears the turret flag and sets the image name to "ZZS", and the art section "ZZS" in the art database copy, which sets the voxel
    flag. The test expansion archive has the models ZZS, ZZSTUR and ZZSBARL with their motion files,
    copies of clearly different stock models. The override section for V names "ZZS" as share
    source. Expected, by claims 16, 18 and 30:
    - infantry and vehicle, turret share flag: the ZZSTUR model (the infantry type has a turret part,
      the vehicle type without the turret flag has none). Falsified if the debris is invisible.
    - vehicle and aircraft, body share flag, with ZZSBARL's motion file replaced by an empty file:
      the ZZS model (the aircraft type looks for the barrel, finds it invalid and loses its voxel art,
      the vehicle type does not look for a barrel). Falsified if the debris is invisible.
    - aircraft and building, body share flag, all files valid: the ZZS model (a building has no body
      model). Falsified if the debris is invisible.
  - Observable: the debris model. Runs: one per variant.
- **A23 — superweapon sidebar icon (claim 31).**
  - Setup: a temperate map, superweapons enabled in the skirmish settings, on which the player owns
    a test building explicitly configured to grant the listed test superweapon, with a supplied
    initial icon, so that its icon is on the sidebar.
  - (a) The map's override section for the superweapon sets its sidebar icon name to a second
    invented name with a distinguishable supplied icon. Expected: the other superweapon's icon. Falsified if the usual icon shows.
  - (b) It sets the name to an invented name with no file. Expected: the placeholder icon. Falsified
    if the usual icon or no icon shows.
  - (c) It sets only an unrelated key. Expected: the usual icon. Falsified otherwise.
  - Observable: the sidebar icon. Runs: one per variant.

- **A24 — art-file trailer sections (claim 33; Q-0083).**
  - Setup: named test data, an art database copy in the test expansion archive. Tiny temperate
    and snow skirmish maps, game speed 2, 800×600, director and mods off except the named test
    files; one human player and one bystander computer player kept outside the observed firing
    and flight corridor. Add an aircraft and a visible long-flight projectile first in the map
    layer, each with an explicit image name different from its type name and the arctic flag set.
    Supply distinct, already listed visible trailer animations in the type-name art section,
    the explicit-image art section, the image-plus-A art section and the rules section.
    Provide valid body art, including an arctic model for the voxel-projectile variant.
  - Procedure: fly the aircraft through the corridor and fire the projectile across it. Record
    video with tick numbers and trail pictures. One run per variant, several flights or shots.
    Repeat with sprite and voxel art. Swap the two image-section trailers as a positive control.
  - Expected: a sprite aircraft uses the suffixed trailer on its first snow read, unsuffixed
    on temperate. A voxel aircraft and either projectile use the unsuffixed trailer on both.
    Neither uses the type-name or rules trailer. A control must show the swapped trailer.
    A contrary trailer falsifies section selection; no trail with a failed control is inconclusive.
  - Delay test: give the selected and rejected sections different positive integer delays.
    Use a non-scalable projectile and measure consecutive trail-creation ticks as in S-0306 P10.
    Expected: the selected unsuffixed art section supplies the delay, not the rules or suffixed
    section. A period matching either rejected section falsifies. For the aircraft, compare
    each flight with controls in which all candidate sections give the same delay: one control
    per candidate delay. Controls must have distinguishable trail intervals. Expected: the
    test matches the selected-section delay's control, and changing a rejected section's delay
    does not alter it. Matching a rejected delay's control falsifies; indistinguishable controls
    are inconclusive. This comparison asserts section selection, not an aircraft timing formula.
    For either kind, a first read without a delay must match an explicit 3-tick-delay control.
  - Layered test: a named loose base-rules copy gives the projectile a non-empty image and
    trailer settings in the matching art section. The map's override omits the image, then in a
    separate variant sets it empty. Expected: body art disappears but the earlier trail and
    period remain. Restating a new non-empty image selects that image's trailer and delay.
    In that new art section, absent or empty trailer retains the earlier trail, a reserved
    no-type value clears it, and a missing or empty delay retains the earlier delay. Test each separately,
    with an explicit-image positive control. A lost or changed trail merely because the image
    is omitted, a trail after explicit clearing, or an incorrect retained period falsifies.
  - Building section selection and byte limits: run S-0004 K2 and K7. Their marked overlay
    artwork distinguishes the image-named, suffixed and type-name sections and the 24-byte cut.

## Open questions

- Which parts of a voxel projectile's model are drawn (claim 27), and how a voxel animation's model is
  drawn without a motion file (claim 30): rendering, S-0105.
- The on-screen meaning of the barrel role for building voxel models (claim 20): rendering,
  S-0105. Alternate-body selection is already owned by S-0105 claim 12, not an open lookup rule.
- What the original draws for a voxel animation whose own model (I ‖ .vxl) cannot be resolved or is
  invalid (claim 30). Our engine's result there is the departure that claim 30 states.
