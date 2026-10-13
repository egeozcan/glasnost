---
id: S-0007
title: Art lookup (image indirection, sprite or voxel, companion files, theater substitution)
topic: rules-import
revision: 8
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-13
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
planned S-0109; a translucent sprite is drawn as a light-glow image, S-0104 and S-0110).

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
  no art file of their own (claim 32). The rules layers of a game are the base rules, the language rules file if a valid
  one exists, the game-mode layer and the map layer (S-0006). A type has a type read in a rules layer
  when it exists by the time its kind's reading pass runs in that rules layer (S-0004 claim 6) and
  the rules layer has a section for it. A type that a rules layer creates by a reference that S-0004
  claim 6 leaves unread in that rules layer (for example an animation first named by a building,
  vehicle, aircraft or infantry type, or by a late game-wide section) therefore has no type read in
  that rules layer (S-0006 claim 7).
  - **Animations** are read from the art database in every rules layer (S-0006 claim 8), so an
    animation with an own art section has a type read in every rules layer in which it already
    exists when the animation pass runs, whatever that rules layer's own file holds.
  - The game-mode layer is applied in every game that is not a campaign mission, except a game at
    whose start the language rules file fails its check (S-0006 claim 12). A mode whose
    game-mode file is not named or cannot be resolved contributes an empty rules layer (S-0005 claim
    19), which still gives every existing animation a type read.
  - So, under the measurement setup (no language rules file), an animation that the base rules'
    animation list names has **three** type reads in every skirmish or multiplayer game and **two**
    in a campaign mission, and one more in each with a valid language rules file. An animation
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
  image (S-0104), not with the translucency codes of S-0109 (animation playback, planned).
- **Infiltrated factory.** A house *has infiltrated a barracks* once, in the current game, one of its
  infiltrating units has entered another house's building that produces infantry, and *has
  infiltrated a war factory* once one has entered another house's building that produces vehicles
  (naval vehicles included).
  How infiltration happens belongs to a special-abilities spec; this spec only uses the result.

## Behaviour

### Image name and art section

1. **Image name.** A type's image name is the value of the image key in the type's own section of the
   rules database. At a type's first type read in a game the default is the type name. At a later type
   read in the same game, a missing image key keeps the image name already set (claim 10 gives a
   consequence). An empty value counts as missing (S-0003), so the image name is never empty.
   Projectiles differ (claim 26). Values
   longer than 24 bytes are cut to their first 24 bytes. For **animations**, the image key
   is read from the own art section, not from the rules database.
   - Backing: `measurable` (A4; S-0004 K7 for byte rather than decoded-character limits)
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
   - Backing: `measurable` (A9)
   - Confidence: high (the table), medium (the number of type reads per game, Terms)

3. **Image keys inside the art database.** An art section can itself hold an image key. It is
   honoured only for **buildings**, where it names the body sprite file (claim 11). For
   **animations** the image key that counts is the one in the own art section (claim 1). An image key
   in the art section named by an animation's image name is never used. For infantry, vehicles and
   aircraft an image key in the art section is ignored: their file names come from the image name of
   claim 1. Stock data contains such unused keys.
   - Backing: `measurable` (A4 for infantry, A9 for animations, A12 part b for buildings)
   - Confidence: high

4. **Sprite or voxel model.** Infantry, vehicles and aircraft use a voxel model when their art section's
   voxel flag is set (default: not set), and a sprite file otherwise. A type with the voxel flag set
   never loads a body sprite, even if its voxel model is missing (claim 18). Buildings, animations,
   overlays, terrain objects and smudges always use sprite files for their main art. Buildings can
   add voxel turret parts (claim 20). Projectiles are drawn as a voxel model when their art section's
   voxel flag is set and as a sprite otherwise (claim 27). Particles and particle systems use sprite
   files only, and have no sprite and no model when their art section sets the voxel flag (claims 28,
   29).
   A voxel animation always uses a voxel model (claim 30).
   - Backing: `measurable` (A5 part h)
   - Confidence: high

### Sprite file names

5. **Base sprite name.** The sprite file of an infantry, vehicle, aircraft, overlay or animation type
   is image ‖ .shp, changed by claims 6 to 10. The same holds for particles, particle systems,
   voxel animations and projectiles, with the differences that claims 26 to 30 state.
   - Backing: `measurable` (A1, A4)
   - Confidence: high

6. **Theater flag: theater extension.** For the kinds of claim 5, when the art section's theater flag
   is set (for an animation, the theater flag as claim 2 gives it), the sprite file is image ‖ .E instead, where E is the current theater's extension (for
   example ".tem" in the temperate theater). Letter substitution is not applied to such a name.
   Buildings differ (claim 11).
   - Backing: `measurable` (A2)
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
   exactly), and the sprite name is built from the suffixed name (so image "X" loads "XA.shp"). The
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
   - Confidence: high (the suffix and the sprite, the section an animation's flag comes from),
     medium (which settings come from which section)

10. **When the arctic suffix is added.** Only snow games are affected. At each type read after which
    the alternate-arctic-art flag is set, an A is appended to the image name as that read leaves it,
    unless the flag was also set after the type's previous type read in the same game. The first type
    read of a game counts as following a read with the flag not set. A read after which the flag is
    not set appends nothing and removes nothing. So an A is appended exactly at the reads where the
    flag goes from not set to set, and at the first read if the flag is set there. Whether the
    previous read appended an A does not matter. Observable consequences within one game:
    - A later type read (game-mode file or map override section, S-0006) that **has an image key**
      replaces the image name with that key's value, whatever the value is (even the same text as the
      rules database's), and the result gets **no** A while the flag stays set. On snow the type then
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
      63 bytes. These are byte limits, not decoded-character limits. S-0108's Import mapping
      currently describes related building-sprite name limits as characters; those descriptions
      must be changed to bytes in the same review round. The image key is read afresh at every type
      read from the art section as it is then: when that art section does not set it, that read uses
      the image name, whatever an earlier read's art section had set.
    - The build-up name is cut to **15** bytes, not decoded characters; the corresponding S-0108
      Import mapping limit must be changed to bytes in the same review round. A type read whose art
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
      e for the build-up sprite, A19 for the eight other sprites at a later type read)
    - Confidence: high (names and substitution, the later-read table), medium (the 63- and
      15-character cuts)

12. **Building art at the theater refresh.** At the theater refresh of every game, a building with the
    theater flag reloads its body sprite as image ‖ .E and its build-up sprite as build-up name ‖ .E.
    This refresh ignores the art section's own image key, applies no letter substitution and has no
    generic fallback. If the body file cannot be resolved the building has no body sprite, and if the
    build-up file cannot be resolved it has no build-up sprite, even if the type read of claim 11 had
    found one. A building with the new-theater flag (and not the theater
    flag) rebuilds all its sprites exactly as in claim 11, which gives the same names again. A
    building with neither flag keeps the result of claim 11. Stock data has no building with the
    theater flag.
    (`quirks.theater_refresh_building_no_fallback`)
    - Backing: `measurable` (A2 for the new-theater case, A12 for the theater-flag case)
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
    - Backing: `measurable` (A2, A11, A13)
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
    the one seen, not the type read's. Stock data uses both flags on animations: the new-theater flag
    on building animations and the theater flag on terrain-like animations.
    (`quirks.theater_art_uses_type_name`)
    - Backing: `measurable` (A8, A13)
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
    - Backing: `measurable` (A5)
    - Confidence: high

17. **Multiple-turret count.** The rules count n of claim 16 selects n turret models. The original
    supports a multiple-turret count of at most 18. With a count above 18 its behaviour is
    *unspecified*. Our engine treats such a count as 18, and the importer warns.
    - Backing: `unmeasured` (the original's result above the limit is unspecified, so there is nothing
      stable to measure)
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
    - Backing: `measurable` (A5)
    - Confidence: high (missing body, missing numbered turret, missing optional parts, invalid motion
      file), medium (a missing motion file next to a found model, A5 part i; that nothing is drawn
      after a discard)

19. **Sprite body with a voxel turret.** A vehicle that uses a sprite body (voxel flag not set) and has
    the turret flag also looks for a voxel turret I ‖ "TUR" and barrel I ‖ "BARL" (single parts only,
    no numbered parts), provided its body sprite loaded. Each part loads on its own: a missing or
    invalid part never removes the body sprite or the other part, and nothing is discarded.
    - Backing: `measurable` (A14) for the body sprite staying whatever the turret files are.
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
    absent. Stock defences rely on the last row: their only model is in the barrel role. Unlike
    vehicle voxels (claim 18), nothing is discarded here: each part loads or not on its own.
    (`quirks.building_turret_without_tur_in_barrel_role`, `quirks.building_tur_search_skips_four`)
    - Backing: `measurable` (A10)
    - Confidence: high (which files load), low (what the barrel role means on screen, S-0105)

21. **No separate shadow files.** No object kind loads a separate shadow file. A sprite's shadow is
    drawn from image frames inside the same sprite file, and
    a voxel shadow is derived from the model (S-0105). Which image frames hold the shadow is
    given for infantry by S-0107 and for buildings by S-0108; for animations it will be given by
    S-0109 (animation playback, planned).
    For overlays, terrain objects, smudges, projectiles and particles no spec gives it
    yet; this claim says only that no separate file is loaded for them. It holds for every kind in
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
    image name. When the sidebar shows the main icon is claim 25.
    (`quirks.main_icon_first_read_only`)
    - Backing: `measurable` (A16)
    - Confidence: high (placeholder, first-read freeze), medium (the 24-character cut)

23. **Translucent sprite.** If a type's own section sets a translucent-sprite name, the file
    translucent-sprite name ‖ .shp is loaded, with no theater handling and no generic fallback. The
    name is cut to its first 24 bytes. The section is the one of claim 1: the type's section of
    the rules database, or, for an animation, its own art section (so a rules layer cannot change an
    animation's translucent sprite, S-0006 claim 8). The loaded sprite is the type's light-glow image
    of S-0104 (Terms, claim 15) and S-0110 (claim 6), which changes the light-alpha values under it
    instead of being painted. A building shows it, and so does an animation, from its creation. This
    spec makes no claim about which other kinds show it, or when. It is unrelated to an animation's
    translucency codes (planned S-0109), which only make the animation's own sprite see-through. Where the glow is placed and how it writes
    the values is S-0104's and S-0110's.
    - Backing: `measurable` (A17; part d for an animation's translucent sprite)
    - Confidence: medium (high for the section an animation's key comes from; medium that an
      animation's glow shows, since the glow's own drawing is S-0104's)

24. **When names are built.** Every game creates the types of the kinds this spec covers afresh after
    the game's theater is set (Terms, *type read*).
    Sprite and voxel names are built and loaded at the type reads of the game (base rules, language
    rules file if a valid one exists, game-mode layer except in campaign missions and in a game at
    whose start the language rules file fails its check, map layer; Terms,
    *type read*), and the theater refresh follows after all of them. The refresh does not touch
    projectiles, particles, particle systems, voxel animations or superweapon icons: for them the
    last type read's result is the one seen in play. The observable consequences
    of this order are listed under State and ordering. All names resolve through
    S-0005, so they match without regard to letter case. Which of these loads go through the
    per-session store, and as which sort (kept file, deferred sprite or art read afresh), is S-0005
    claim 22; this spec makes no claim about it.
    - Backing: `measurable` (A1, A4, A8)
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
    (`quirks.alt_icon_kept_once_loaded`)
    - Backing: `measurable` (A18)
    - Confidence: high (the loading rules, the three conditions), medium (that the choice follows the
      infiltration at once)

### Projectiles, particles, particle systems, voxel animations, superweapons

26. **Projectile image name.** After a type read of a projectile, its image name is the value of the
    image key in that read's section of the projectile (cut to 24 bytes), and **empty** when that
    section has no image key or an empty one. Unlike claim 1, there is no default of the type name and
    nothing carries over from an earlier type read. A projectile with an empty image name has no
    sprite and no model; its translucent sprite still follows claim 23. (Its lookups would use names with an empty base, such as ".shp"; no stock
    file has such a name, and our engine gives it no art.)
    Consequences:
    - Stock data has projectiles without an image key that are not invisible projectiles. They show
      no sprite in flight.
    - A later type read of a projectile (a game-mode file or map override section for it, S-0006)
      that has no image key leaves it without art for the rest of the game, even if the base rules
      gave it an image key. The importer warns when such a section has no image key while an earlier
      rules layer gave one.
    (`quirks.projectile_image_afresh`)
    - Backing: `measurable` (A20 parts a and b; S-0004 K7 for the byte limit)
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
    - Backing: `measurable` (A20 parts b to f)
    - Confidence: high (sprite names, voxel parts), medium (the arctic results)

28. **Particles.** A particle's image name follows claim 1 (its rules section; default the type
    name; a later type read without an image key keeps it). Its sprite follows claims 5 to 10 like an
    infantry sprite: theater extension, letter substitution with the new-theater flag, generic
    fallback and arctic suffix. A particle whose art section sets the voxel flag has no sprite and
    no model; its translucent sprite still follows claim 23. The theater refresh does not touch
    particles.
    - Backing: `measurable` (A21)
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
      matches more than one kind. S-0004 claim 13 currently says that this priority is unmeasured
      because no observable in that spec distinguishes it; that rationale must be revised to
      acknowledge S-0007's A22 part e in the same review round.
    - Confidence: high (the file name, the share order and priority), medium (no motion file, the
      afresh share keys)

31. **Superweapon sidebar icon.** A superweapon's sidebar icon is c ‖ .shp, where c is the sidebar
    icon name in the superweapon's rules section, cut to 24 bytes. Its default is the type
    name, and a later type read without the key keeps the name (an empty value counts as missing,
    S-0003). There is no theater handling and no generic fallback. If the file cannot be resolved,
    the placeholder "XXICON.SHP" is used, as for the main icon (claim 22). Unlike the main icon, the
    superweapon icon is built again at **every** type read, so a game-mode file or map override
    section can change it.
    - Backing: `measurable` (A23)
    - Confidence: high

32. **Kinds without art files of their own.** Weapons, warheads and tiberiums load no art file of
    their own, through an image name or any other name. Weapons and warheads only name other types
    (animations, particle systems, projectiles), whose art follows those types' own claims. A
    tiberium's image key is a number that selects its overlays, not a file name (a tiberium spec
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
  "type name" is the type's type name.
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

> theater flag set: file = found(I ‖ .E)
> new-theater flag set (theater flag not set): file = found(subst(I ‖ .shp))
> neither: file = found(I ‖ .shp)

**Building body sprite**, at each type read, where J is the art section's own image key when that
key is set, and I otherwise:

> theater flag set: file = found(subst(J ‖ .E))
> otherwise: file = found(subst(J ‖ .shp))

**Building body sprite at the theater refresh**, theater flag set (claim 12):

> file = I ‖ .E when that resolves; = ∅ otherwise

**Animation sprite**, at each type read:

> theater flag set: file = found(I ‖ .E)
> otherwise: file = found(I ‖ .shp), with no subst

**Animation sprite at the theater refresh:**

> theater flag set: file = (type name) ‖ .E when that resolves; = (type name) ‖ .shp when (type name) ‖ .E
> does not resolve and (type name) ‖ .shp does; = ∅ otherwise
> new-theater flag set (theater flag not set): file = found(subst(I ‖ .shp))
> neither: the sprite of the last type read is kept

**Terrain object and smudge sprite** (no generic fallback anywhere):

> theater flag not set: file = I ‖ .shp when that resolves at the type read; = ∅ otherwise
> theater flag set: file = (type name) ‖ .E when that resolves at the theater refresh; = ∅ otherwise

**Projectile** (claims 26, 27), with K the image key of the type read's section (cut to 24
bytes) and I = K, or empty when K is absent or empty:

> I empty, or invisible projectile: no art
> voxel flag not set: sprite as the infantry sprite above, from I (never I ‖ "A")
> voxel flag set: model parts of claim 16 from I, or from I ‖ "A" at a type read where claim 10
> appends an A (snow only)

**Particle and particle system** (claims 28, 29): the infantry sprite formulas above, with claim 1's
image name; no art when the art section's voxel flag is set.

**Model of a voxel animation** (claim 30), at each type read:

> any share flag set: the share source's part named by the highest-ranking share flag set (body
> above turret, turret above barrel), with its motion file,
> or ∅ when there is no share source or it lacks that part
> otherwise: model = I ‖ .vxl, no motion file, no fallback

**Superweapon icon** (claim 31), at every type read, with s the sidebar icon name cut to 24
bytes:

> s ‖ .shp when that resolves; = XXICON.SHP otherwise

**Icons** (claims 22 and 25), with c and a the icon name and the alternate icon name, each cut to 24
bytes:

> main icon, set at the first type read only: c ‖ .shp when c is non-empty and
> c ‖ .shp resolves; = XXICON.SHP otherwise
> alternate icon, at every type read while none is loaded: a ‖ .shp when a is non-empty and
> a ‖ .shp resolves; = ∅ otherwise

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
| image name | type read with an image key | the key's value, cut to 24 bytes |
| image name | type read without an image key | unchanged |
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

For an animation, only type reads after which its theater flag is set take part in the three arctic
rows. Its other type reads append nothing and leave the arctic state unchanged (claims 9, 10).

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
  substitution. Alternative: only with the new-theater flag. Stock buildings without the flag depend
  on the YR behaviour (their per-theater sprite files exist under substituted names), so the
  alternative breaks stock data.
- **`quirks.theater_art_uses_type_name`** (claims 13, 14). YR: terrain objects, smudges and
  animations with the theater flag build their refreshed file from the type name. Alternative: use the
  image name. In stock data the image name equals the type name for all of these, so both give the
  same files.
- **`quirks.theater_refresh_building_no_fallback`** (claim 12). YR: the theater-flag refresh of a
  building has no generic fallback and ignores the art section's image key, and a miss leaves the
  building without a body sprite. Alternative: keep the type read's result.
- **`quirks.apc_water_body`** (claim 16). YR: only a vehicle whose type name is exactly "APC" looks for
  the I ‖ "W" alternate body. Alternative: every vehicle with the voxel flag and without the turret
  flag looks for it, whatever its type name. Stock data has no vehicle type with the type name "APC" and no
  vehicle whose image name ‖ "W" names a stock model, so the choice does not change stock games.
- **`quirks.voxel_part_failure_discards_all`** (claim 18). YR: a missing numbered turret, or any
  found but invalid part (or its motion file), discards the whole voxel art of the type, and nothing
  of it is drawn. Alternative: only that part is absent; only a missing or invalid body removes the
  type's voxel art.
- **`quirks.building_turret_without_tur_in_barrel_role`** (claim 20). YR: a turret-animation name
  without a late "TUR" (and no barrel-animation-is-voxel flag) becomes the barrel model. Alternative:
  load it as the turret model.
- **`quirks.building_tur_search_skips_four`** (claim 20). YR: "TUR" counts only from the fifth
  character of the turret-animation name. Alternative: an occurrence anywhere in the name counts. No
  stock name is affected.
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
  but must determine whether its name resolves. Backing: none needed; the type-read picture is
  never visible, so this entry is informative only.
- **Unused art-section image keys** (claim 3). For infantry, vehicles and aircraft an importer must
  ignore an image key inside the art section. Using it would change which file stock types load.
  Backing: `measurable` (A4).
- **Turret count above 18** (claim 17) and **invalid numbered barrel** (claim 18): the original's
  result is unspecified. Our engine clamps the count to 18 with an importer warning, and discards the
  voxel art for an invalid numbered barrel. These are departures, not quirk switches.
- **`quirks.projectile_image_afresh`** (claim 26). YR: a projectile's image name is its type read's
  image key or empty, so a later type read without the key leaves the projectile without art.
  Alternative: a later type read without an image key keeps the image name of the earlier read (the
  first read still gives an empty name without a key, so stock projectiles without a key stay
  without art). With either setting the importer warns about an override section that drops the
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

All test data is removed afterwards. Test sprite and model files are copies of stock files, chosen so
that the pictures to tell apart are clearly different (for example, two infantry of different
sides). Every scenario is a skirmish on the test map, with the objects owned by the player and in
view, except A9 (iii-b), which says so. Scenarios that count type reads use types that the base rules'
type lists name, so that the counts of Terms (*type read*) hold. Art lookup is deterministic, so each scenario needs **one run** per map and per variant. The
observable is always a screenshot, or the sidebar, and what it shows.

**Pictures compared across theaters.** Each theater has its own unit, tile and theater palettes
(glossary, *palette*), so one sprite file can look different on two theaters. Where a scenario
compares an object's pictures on maps of different theaters, and the palette that object is drawn
with differs between those theaters (S-0103), "the same picture" means the same image frames
whatever their colours, and "different pictures" means a difference that the change of palette
alone cannot produce. A stock object of the same kind, drawn with the same palette, whose file name
does not depend on the theater, placed on the same maps, shows what the palette alone changes. For
example, in A1 a building of part (b) drawn with the same palette as the building of part (a) is
that control, and in A6 part (a) a stock infantry type without the alternate-arctic-art flag, the
theater flag or the new-theater flag is.

- **A1 — building letter substitution without the new-theater flag (claims 5, 7, 11, 24).**
  - Setup: stock data. Four tiny maps, temperate, snow, urban and desert, each with the same stock
    building placed on it. Choose a building whose art section does not set the new-theater flag and
    whose image name letter substitution changes (claim 7): first letter G, N, C or Y and second
    letter A or T.
  - Procedure: start each map, also as the first game of a fresh session, and screenshot the
    building.
  - Observable: the building's picture on each map.
  - Expected: four different, theater-specific pictures.
  - Falsified if all four show the same picture.
  - (b) Negative case of claim 7. Setup: the same four maps with a stock building without the
    theater flag whose body sprite name (its art section's own image key if set, else its image
    name) does *not* start with a pair of claim 7 (the first letter is not G, N, C or Y, or the
    second is not A or T). Observable: its main picture on each map. Expected: the same main
    picture on all four. Falsified if any two differ.
- **A2 — theater extension and refresh for stock kinds (claims 6, 11, 12, 13, 15).**
  - Setup: stock data. The same four maps, each with: a stock tree (terrain object with the theater
    flag), a stock overlay with the theater flag, a stock smudge with the theater flag, and a stock
    building with the new-theater flag that has a build-up sprite.
  - Procedure: start each map, screenshot all four objects, and watch the building's build-up when a
    copy of it is placed.
  - Observable: each object's picture and the build-up animation on each map.
  - Expected: theater-specific pictures for all of them on every map where stock data has the
    theater's files.
  - Falsified if the tree, overlay or smudge is missing on a theater that has its file (type name ‖
    .E for the tree and the smudge, image ‖ .E for the overlay), or if the building or its build-up looks the same on all four theaters.
  - (b) No letter substitution on theater-extension names (claim 6). Test data, temperate map: in
    the art database copy, a section "GAZZO" copied from the art section of a stock overlay with the
    theater flag; in the map, an override section setting that overlay's image key to "GAZZO". In
    the test expansion archive, "GAZZO.tem" (picture P) and "GTZZO.tem" (picture Q, a clearly
    different overlay picture). Observable: the overlay's picture. Expected: P. Falsified if Q shows.
- **A3 — generic fallback (claims 7, 8).**
  - (a) Setup: stock data, a temperate map with a stock building whose image name letter substitution
    changes, for which stock data has a snow sprite and a generic (second letter G) sprite but no
    temperate sprite. A snow map with the same building as reference. Expected: on temperate the
    building is visible with the generic picture, which differs from the snow picture. Falsified if
    it is invisible on temperate, or shows the snow picture.
  - (b) Test data: in the art database copy, a section "FOO" copied from a stock infantry's art
    section. In the test expansion archive, a sprite "FGO.shp" (a copy of a different infantry's
    sprite) and no "FOO.shp". In the map, an override section setting that infantry's image key to
    "FOO". Expected: the infantry shows the FGO picture. Falsified if it is invisible.
  - (c) Test data: as (b), with the section and image key "Q" and a sprite file named "QGshp" (no
    dot). Expected: the infantry shows the QGshp picture. Falsified if it is invisible.
  - (d) Theater extension. Test data: in the art database copy, a section "FOO" copied from the art
    section of a stock overlay with the theater flag. In the map, an override section setting that
    overlay's image key to "FOO", on a temperate map where the overlay is placed. In the test
    expansion archive, "FGO.tem" (a copy of a clearly different overlay picture) and no "FOO.tem".
    Expected: the overlay shows the FGO picture. Falsified if it is invisible.
  - Observable: the object's picture. Runs: one per map and variant.
- **A4 — image indirection, unused art-section image keys, letter case, length cut (claims 1, 3, 5,
  24).**
  - (a) Test data: in the art database copy, give a stock infantry's art section an image key that
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
  - Observable: the infantry's picture. Runs: one per variant.
- **A5 — voxel parts (claims 4, 16, 18).** Two kinds of change are used: an *invalid* part is an
  empty file under the part's name in the test expansion archive; a *removed* part uses the loose
  nested-archive copy minus that file. Observable: whether the vehicle is drawn, and which parts it
  shows. Runs: one per variant.
  - (a) A stock single-turret tank: invalid turret motion file (I ‖ "TUR" ‖ .hva). Expected: the tank
    is not drawn at all. Falsified if it is drawn.
  - (b) The same tank: invalid barrel motion file (I ‖ "BARL" ‖ .hva). Expected: not drawn.
    Falsified if drawn.
  - (c) The same tank: barrel model removed. Expected: drawn, without a barrel. Falsified if not
    drawn.
  - (d) A stock vehicle with a multiple-turret count of at least 2 and no gatling flag: turret 1
    model (I ‖ "TUR1") removed. Expected: not drawn. Falsified if drawn.
  - (e) The same vehicle: barrel 1 model (I ‖ "BARL1") removed, if stock data has it. Expected:
    drawn. Falsified if not drawn.
  - (f) A stock vehicle without the turret flag that sets the no-spawn-alternate flag and has an I ‖
    "WO" model in stock data: invalid I ‖ "WO" ‖ .hva. Expected: not drawn. Then instead remove its I
    ‖ "WO" model. Expected: drawn. Falsified by either other outcome.
  - (g) Test data: the map lists a new vehicle type "APC" with a full definition copied from a stock
    vehicle without the turret flag and without the no-spawn-alternate flag, and a second new type
    "APD" defined the same way (S-0006 claim
    7). The art database copy has sections "APC" and "APD" with the voxel flag set. The test
    expansion archive has valid body files APC and APD (.vxl and .hva copies of a stock vehicle), and
    APCW.vxl and APDW.vxl with *empty* motion files APCW.hva and APDW.hva. Place one of each.
    Expected: "APC" is not drawn and "APD" is drawn. Falsified if both are drawn or neither is.
  - (h) Test data: the stock tank of (a) with its body model removed, and a body sprite I ‖ .shp
    (copy of a stock vehicle sprite) added. Expected: not drawn. Falsified if the sprite shows.
  - (i) The tank of (a): its turret motion file (I ‖ "TUR" ‖ .hva) removed, the turret model kept.
    Expected: not drawn. Falsified if drawn.
- **A6 — arctic alternate (claim 9).** Setup: stock data, plus test data in variants (b) and (c). A snow map
  and a temperate map, each with a stock infantry type that sets the alternate-arctic-art flag,
  buildable from the sidebar.
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
    image name (its stock image key's value, or its type name if it has none). A second
    map has none. Expected: the arctic picture without the override, the
    non-arctic picture with it. Falsified if both show the arctic picture.
  - (b) The override section sets another key unrelated to art, and no image key. Expected: the
    arctic picture. Falsified if it shows the non-arctic picture.
  - (c) For a stock infantry that does not set the flag: the override section sets the flag, and the
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
  - Setup: stock data. Tiny temperate, snow and desert maps, each with a stock building whose active
    animation has the new-theater flag and an image name that letter substitution changes, and a
    stock theater-flag animation W (a waterfall or tunnel top) placed on the map. Both animations
    are named by the base rules' animation list.
  - Procedure: start each map, including as the first game of a fresh session, and screenshot the
    building's animated part and the waterfall.
  - Observable: their pictures per theater.
  - Expected: theater-specific pictures in every theater, including the first game.
  - Falsified if the building's animation shows the same picture on all three theaters, or if W
    shows the same picture on two theaters for which stock data has different W type name ‖ .E
    files, or is invisible on a theater for which stock data has W's type name ‖ .E file.
  - Optional test data (claim 14, refresh by type name with the .shp retry), temperate map only. Two
    conditions make it discriminate: the image name must differ from the type name, or the
    image-named and type-name-named files would be the same files; and the theater flag must still
    be set after the game's last type read (claim 2), or the refresh would not take the theater-flag
    path. So: in the art database copy, W's own art section gets an image key naming an invented
    name "ZZW", and the copy has **no** section "ZZW", so every read takes the theater flag from W's
    own art section (claim 2). In the test expansion archive: "ZZW.tem" (picture P) and W's type
    name ‖ .shp (picture S, a copy of a clearly different sprite). W's type name ‖ .tem is made
    absent with a loose copy of the stock nested archive that holds it, minus that file (Common
    setup). Expected: S. Falsified if P shows, if W's usual picture shows, or if W is invisible.
  - Runs: one per map.
- **A9 — animation flags from two sections (claims 2, 3).**
  - Setup: test data on a temperate map with a stock building whose active animation X has the
    new-theater flag in its own art section. In the art database copy: X's own art section gets the
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
    with a stock theater-flag animation X placed on the map (as in A8), named by the base rules'
    animation list and chosen so that stock data has X ‖ .tem (its usual picture, R′). Neither of the sections below sets the new-theater flag.
    In the art database copy: X's own art section gets the image key "ZZQ", and a new section "ZZQ"
    sets the theater flag to *no*. In the test expansion archive: "ZZQ.tem" (picture P′, a clearly
    different sprite). No "ZZQ.shp" and no "ZGQ.shp" exist.
    - (iii-a) A skirmish. Every skirmish gives X three type reads (base rules, game-mode layer, map
      layer; Terms, *type read*), whether or not its mode has a game-mode file. Read 1 is the first, so it takes the own
      section's theater flag and finds ZZQ.tem. Read 2 follows a read that found the sprite file, so
      it takes the "ZZQ" section's cleared flag and finds nothing. Read 3 follows a read that found
      nothing, so the theater flag is set once more, and the refresh loads X ‖ .tem. Expected: R′. Falsified if the animation is invisible or shows P′.
    - (iii-b) The same test map and test data, played as a campaign mission instead of a skirmish
      (the exception in Common setup): a mission map file of the test map's content, with X placed
      in view, stands in for a stock mission's map as a loose file of that name (S-0005 claim 1). A
      campaign mission has no game-mode layer, so X has two type reads (base rules, map layer). The
      last read leaves the theater flag cleared and no sprite. Expected: the animation is
      invisible. Falsified if R′ or P′ shows.
    - In general the hypothesis predicts R′ when the game has an odd number of animation type reads
      and nothing when it has an even number. Since (iii-a) and (iii-b) differ by one read, the
      hypothesis is also falsified if both variants give the same result, whatever the read count.
      If, instead, the second row of claim 2 applied to every read after any read that had found
      the sprite file, both variants would show nothing.
    - Observable: the animation's picture. Runs: one per variant.
- **A10 — building voxel turret names (claim 20).** Test data. In the map, override sections give a
  stock defence that sets the turret-animation-is-voxel flag (and not the barrel flag) the
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
  - (a) Setup: stock data. Temperate and snow maps with a stock crater smudge and a stock burn smudge
    (both without the theater flag). Expected: drawn, with the same picture on both theaters.
    Falsified if invisible.
  - (b) Test data: in the art database copy, clear the theater flag of a stock tree. In the test
    expansion archive, image ‖ .shp (a copy of a different tree's temperate picture). Temperate and
    snow maps with the tree. Expected: the copied picture on both maps. Falsified if the tree is
    invisible or theater-specific.
  - (c) As (b) but the file is named with G as second character instead (generic name) and image ‖
    .shp is absent. Expected: the tree is invisible (no fallback). Falsified if it shows.
  - Observable: the object's picture. Runs: one per map and variant.
- **A12 — building art-section names and the theater-flag refresh (claims 11, 12).**
  - (a) Test data: in the art database copy, give a stock building the theater flag and an art-section
    image key J naming an invented file name whose first two letters are not a pair of claim 7. In
    the test expansion archive, J ‖ .tem (a copy of the building's picture). No file image name ‖
    .tem exists.
  - Observable: whether the building's main picture is drawn on a temperate map.
  - Expected: no main picture, because the refresh looks only for image name ‖ .tem, although the
    type read found J ‖ .tem. Then add image name ‖ .tem (a clearly different picture): that picture
    shows.
  - Falsified if the J picture shows in the first run, or if the added image name ‖ .tem picture
    does not show in the second.
  - (b) Test data: in the art database copy, give a stock building *without* the theater flag an
    art-section image key J (as above) and add J ‖ .shp (a clearly different picture) to the test
    expansion archive. Expected: the building shows the J picture. Falsified if it shows its usual
    picture.
  - (c) Length cut of the art-section image key. As (b), but J is an invented name of exactly 63
    characters, the image key holds J followed by five more characters, and the file in the test
    expansion archive is J ‖ .shp. Expected: the J picture. Falsified if the building has no main
    picture.
  - (d) Length cut of the build-up name. Test data: in the art database copy, set the build-up name
    of a stock building without the theater flag to B followed by four more characters, where B is
    an invented name of exactly 15 characters whose first two letters are not a pair of claim 7. In
    the test expansion archive, B ‖ .shp (a copy of a clearly different building's build-up sprite).
    Procedure: place the building. Expected: the copied build-up plays. Falsified if no build-up or
    the usual one plays.
  - (e) Afresh image key, kept build-up name. Test data: a stock building without the theater flag
    whose art section sets its own image key (in the art database copy, give it one if no stock
    building has one) and a build-up name. In the art database copy, a section "ZZB" copied from
    that art section without the image key and without the build-up name. In the map, an override
    section sets the building's image key to "ZZB". In the test expansion archive, "ZZB.shp" (a
    clearly different building picture). Procedure: place the building. Expected: the building shows
    the ZZB picture, and its usual build-up plays. Falsified if it shows the picture named by the
    first image key, or if no build-up plays.
  - Runs: one per variant.
- **A13 — refresh by type name (claims 13, 14).** Test data: on temperate, override sections in the
  map set the image key of a stock theater-flag tree and of a stock theater-flag smudge to the image
  name of another stock tree and smudge with the theater flag. Animations ignore map sections (S-0006
  claim 8), so for them the art database copy gives the own art section of a stock theater-flag
  animation an image key naming another stock theater-flag animation. All three types are named by
  the base rules' type lists. Observable: their pictures.
  Expected: each shows its own usual picture. Falsified if it shows the other object's picture.
  Runs: one.
- **A14 — sprite body with a voxel turret (claim 19).**
  - Test data: in the art database copy, clear the voxel flag of a stock single-turret tank, and add
    its body sprite I ‖ .shp (a copy of a stock vehicle sprite) to the test expansion archive.
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
  - (b) Test data: in the map, an override section sets that unit's image key (with the stock art
    database) to another buildable unit's image name. Expected: the unit in play looks like the other
    unit, but its sidebar icon stays its own. Falsified if the icon changes.
  - (c) Length cut. Test data: in the art database copy, set the unit's icon name to C followed by
    four more characters, where C is an invented name of exactly 24 characters. In the test
    expansion archive, C ‖ .shp (a copy of another unit's icon). Expected: the other unit's icon.
    Falsified if the placeholder shows.
  - Observable: the sidebar icon and the unit's picture. Runs: one per variant.
- **A17 — translucent sprite (claim 23).**
  - Test data: in the map, an override section gives a stock building a translucent-sprite name,
    naming a stock translucent sprite used by another type.
  - (a) On temperate and snow maps. Expected: the same translucent picture on both. Falsified if it
    is missing or differs.
  - (b) The key names an invented file "GAZZL" while the test expansion archive holds only
    "GGZZL.shp" and "GTZZL.shp". Expected: no translucent sprite. Falsified if one shows.
  - (c) The key names an invented name of 24 characters followed by four more, and the test
    expansion archive holds a file named by the first 24 ‖ .shp. Expected: that picture shows.
    Falsified if nothing shows.
  - (d) Animation. Test data: in the art database copy, the own art section of a stock looping
    animation that a stock object plays continuously (a building's idle animation, for example) sets
    a translucent-sprite name naming a stock translucent sprite used by another type; in a second
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
  - Setup: stock data unless stated. A small test map with the player's house, which can build an
    infiltrating unit, and an enemy computer house with a barracks and a war factory within reach.
    The player has a barracks, a war factory and an aircraft factory of their own. Choose, from stock
    data, a trainable infantry type, a trainable vehicle type and an aircraft type that the player
    can build and whose art sections set an alternate icon name naming a stock file. Where stock
    data has no such type of a kind, give one an alternate icon name in the art database copy and
    put the named file (a copy of a clearly different icon) in the test expansion archive.
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
  - (e) A stock building type whose undeploy-into vehicle is set and whose art section sets an
    alternate icon name (test data as in Setup if none). Expected: its icon changes after the war
    factory infiltration and not after the barracks infiltration. Falsified otherwise.
  - Runs: one per variant.
- **A19 — building secondary sprites at a later type read (claim 11).**
  - Setup: test data on a temperate map. Choose a stock building B without the theater flag whose
    art section sets a bib name and no image key of its own. In the art database copy, a new section
    "ZZB" copied from B's art section without the bib name. In the test expansion archive, "ZZB.shp"
    (a copy of B's body sprite, so that the body looks the same). In the map, an override section
    sets B's image key to "ZZB". B is placed on the map.
  - (a) As set up. Expected: B's usual bib is drawn. Falsified if B has no bib.
  - (b) "ZZB" sets the bib name to an empty value. Expected: the usual bib is drawn. Falsified if
    there is no bib.
  - (c) "ZZB" sets the bib name to the invented name "ZZNOBIB", and no file "ZZNOBIB.shp" or
    "ZGNOBIB.shp" exists. Expected: no bib. Falsified if the usual bib is drawn.
  - (d) "ZZB" sets the bib name to the bib name of a clearly different stock building. Expected:
    that building's bib. Falsified if the usual bib or no bib is drawn.
  - Observable: the bib under B. Runs: one per variant.
- **A20 — projectiles (claims 26, 27).**
  - Setup: a temperate map with a unit of the player that fires a slow, clearly visible sprite
    projectile (for example an artillery shell or a missile drawn as a sprite) whose rules section
    sets an image key, and a target. Variants (c) to (e) use a weapon with a voxel projectile or a
    sprite projectile as stated. Screen captures of the projectile in flight.
  - (a) The map's override section for the projectile sets an unrelated key and no image key.
    Expected: the projectile is not drawn in flight (its hit effects still show). A control run
    without the override shows it. Falsified if it is drawn.
  - (b) The override section sets the projectile's image key to the image name of another stock
    sprite projectile. Expected: the other projectile's picture. Falsified if the usual picture
    shows.
  - (c) A stock projectile with the voxel flag: in the test expansion archive, an empty file under
    its image name ‖ .hva. Expected: the projectile is not drawn. Falsified if it is drawn.
  - (d) Snow map, the voxel projectile of (c) without the change: the override section restates its
    image key I and sets the alternate-arctic-art flag. The test expansion archive has I ‖ "A" ‖ .vxl
    and .hva, copies of a clearly different stock model. Expected: the different model on snow, the
    usual model on a temperate control map. Falsified if the snow map shows the usual model.
  - (e) Snow map, the sprite projectile of (a): the override section restates its image key I and
    sets the alternate-arctic-art flag. The test expansion archive has I ‖ "A" ‖ .shp, a copy of a
    clearly different sprite. Expected: the usual picture. Falsified if the different picture
    shows.
  - (f) A stock weapon with an invisible projectile: the override section gives that projectile the
    image key of the sprite projectile of (a). Expected: still nothing drawn in flight. Falsified if
    the sprite shows.
  - Observable: the projectile in flight. Runs: one per variant, several shots each.
- **A21 — particles (claim 28).**
  - Setup: test data on a temperate map with a stock effect that visibly emits a listed particle
    type P (for example smoke from a damaged building or a weapon), triggered in the game.
  - (a) In the art database copy, a section "FOO" copied from P's art section (or empty if P's
    image name has none). In the map, an override section sets P's image key to "FOO". In the test
    expansion archive, "FGO.shp" (a copy of a clearly different particle sprite) and no "FOO.shp".
    Expected: the particles show the FGO picture. Falsified if they are invisible or look as usual.
  - (b) In the art database copy, P's art section (the section named by its image name, added if
    absent) sets the voxel flag. Expected: P's particles are not drawn. Falsified if they are drawn.
  - Observable: the particles. Runs: one per variant.
- **A22 — voxel animations (claim 30).**
  - Setup: test data on a temperate map where the player destroys several stock vehicles whose
    destruction throws a listed, non-sharing voxel animation V (debris such as a tire). Debris is
    thrown at random, so each variant repeats destructions until V has been seen at least three
    times.
  - (a) In the map, an override section sets V's image key to "ZZVA". In the test expansion archive,
    "ZZVA.vxl" (a copy of a clearly different stock voxel animation's model) and **no** "ZZVA.hva".
    Expected: the debris shows the copied model. Falsified if it is invisible or looks as usual.
  - (b) The override section for V sets the body share flag and names a stock vehicle with a voxel
    body as share source. Expected: the debris shows that vehicle's body. Falsified otherwise.
  - (c) A stock voxel animation W that shares a vehicle's turret (thrown when that vehicle is
    destroyed). The override section for W sets the turret share flag again and names a different
    stock vehicle with a voxel turret as share source. Expected: W shows that other vehicle's turret.
    Falsified if it shows the usual turret.
  - (d) Flag priority. The override section for V sets the body and the turret share flags and
    names a stock vehicle with a voxel body and a voxel turret as share source. Expected: the debris
    shows that vehicle's body. Falsified if it shows the turret. Then the section sets the turret and
    the barrel share flags instead, with a stock vehicle that has a voxel turret and a voxel barrel.
    Expected: the turret. Falsified if the barrel shows.
  - (e) Kind priority. Test data: the map's type lists add one new type name "ZZS" to two kinds at a
    time (S-0006 claim 7, as in A5 part g). Both types read the one rules section "ZZS", which does
    not set the turret flag, and the art section "ZZS" in the art database copy, which sets the voxel
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
    a stock superweapon building, so that the superweapon's icon is on the sidebar.
  - (a) The map's override section for the superweapon sets its sidebar icon name to that of another
    stock superweapon. Expected: the other superweapon's icon. Falsified if the usual icon shows.
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

- The one stock terrain type without the theater flag has an image name that names no stock file. It
  is presumably drawn by its own special rules rather than through claim 13. This belongs to the spec
  for that terrain type.
- Which parts of a voxel projectile's model are drawn (claim 27), and how a voxel animation's model is
  drawn without a motion file (claim 30): rendering, S-0105.
- When an alternate body (claim 16) is drawn instead of the body, and the on-screen meaning of the
  barrel role for building voxel models (claim 20): rendering, S-0105.
- What the original draws for a voxel animation whose own model (I ‖ .vxl) cannot be resolved or is
  invalid (claim 30). Our engine's result there is the departure that claim 30 states.
