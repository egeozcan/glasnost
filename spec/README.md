# Specs

Plain-language behaviour specs for the engine. Each spec passed the scrub pipeline in [PROVENANCE.md](../PROVENANCE.md) (sections 2–4) before it was committed here. Implementer sessions read specs; they never edit them, except to add requests under `requests/`.

## How a spec arrives

1. The spec agent drafts it in its own workspace, or answers a request from `requests/`.
2. Lint, then a fresh scrubber session, then sign-off by the project lead or a delegate the lead names (the delegate is named in `approved-by`, and each sign-off is logged on the spec side).
3. The lead commits it as `spec/<topic>/S-xxxx-short-name.md` with `spec: add S-xxxx <title>`.

A spec that contradicts a measurement goes back for revision; the measurement wins. Revisions bump `revision` and go through the same pipeline.

## Layout

| Path | Contents |
|---|---|
| `<topic>/S-xxxx-*.md` | One spec per file, grouped by game topic |
| `TEMPLATE.md` | The spec template |
| `glossary.md` | Our own vocabulary for game concepts |
| `requests/` | Spec requests (`Q-xxxx`) from implementer sessions |
| `incidents/` | Incident logs (`I-xxxx`), see PROVENANCE.md section 8 |

Topics: `simulation`, `combat`, `movement`, `production`, `economy`, `structures`, `infantry`, `vehicles`, `aircraft`, `naval`, `superweapons`, `special-abilities`, `visibility`, `rules-import`, `formats`, `rendering`, `triggers`, `ui`. Add a topic when a spec does not fit; never organise by the original program's modules.

## Index

| ID | Title | Topic | Rev | Status | Measurements |
|---|---|---|---|---|---|
| [S-0001](rules-import/S-0001-ini-lexical-rules.md) | INI lexical rules | rules-import | 3 | approved | — |
| [S-0101](rendering/S-0101-isometric-projection.md) | Isometric projection, height lift, view position and scroll limits | rendering | 3 | approved | — |
| [S-0103](rendering/S-0103-palettes-house-colour.md) | Palettes and house colour | rendering | 3 | approved | — |
| [S-0108](rendering/S-0108-building-sprites.md) | Building sprites | rendering | 3 | approved | — |
| [S-0002](rules-import/S-0002-ini-duplicates.md) | INI duplicates and name resolution | rules-import | 6 | approved | — |
| [S-0003](rules-import/S-0003-ini-value-forms.md) | INI value forms | rules-import | 8 | approved | — |
| [S-0004](rules-import/S-0004-type-registration-order.md) | Type registration order | rules-import | 7 | approved | — |
| [S-0005](rules-import/S-0005-data-file-precedence.md) | Data file precedence (loose files, archive mount order, name lookup) | rules-import | 4 | approved | — |
| [S-0007](rules-import/S-0007-art-lookup.md) | Art lookup (image indirection, sprite or voxel, companion files, theater substitution) | rules-import | 6 | approved | — |
| [S-0011](triggers/S-0011-map-triggers-mp.md) | Map triggers (multiplayer subset) | triggers | 4 | approved | — |
| [S-0106](rendering/S-0106-voxel-tilt-slopes.md) | Slope shapes, terrain surface height and voxel tilt on slopes | rendering | 3 | approved | — |
| [S-0107](rendering/S-0107-infantry-sprite-sequences.md) | Infantry sprite sequences | rendering | 4 | approved | — |
| [S-0109](rendering/S-0109-animation-playback.md) | Animation playback | rendering | 6 | approved | — |
| [S-0110](rendering/S-0110-shroud-rendering.md) | Shroud and fog rendering | rendering | 4 | approved | — |
| [S-0111](rendering/S-0111-in-game-overlays.md) | In-game overlays (health bars, pips, selection marks, insignia) | rendering | 6 | approved | — |
| [S-0201](simulation/S-0201-game-speed-tick-rate.md) | Game speed and tick rate | simulation | 7 | approved | — |
| [S-0204](simulation/S-0204-coordinates-distance-facing.md) | Coordinates, distance and facing | simulation | 5 | approved | — |
| [S-0205](simulation/S-0205-number-domains.md) | Number domains (integers, floating point, the round-toward-zero mode, conversions) | simulation | 4 | approved | — |
| [S-0206](movement/S-0206-turn-rates.md) | Turn rates and turning | movement | 6 | approved | — |
| [S-0210](movement/S-0210-crushing.md) | Crushing (who crushes whom, when, the outcome, walls, warnings, slowdown and automatic crushing) | movement | 11 | approved | — |
| [S-0211](movement/S-0211-land-types-passability.md) | Land types and passability | movement | 7 | approved | — |
| [S-0212](movement/S-0212-pathfinding.md) | Pathfinding (observable behaviour) | movement | 14 | approved | — |
| [S-0213](simulation/S-0213-human-orders.md) | Human orders (the order set, when orders apply, and what each order changes) | simulation | 6 | approved | — |
| [S-0214](ui/S-0214-order-targeting-rules.md) | Order targeting rules (what a click means for each kind of object, and group orders) | ui | 6 | approved | — |
| [S-0215](simulation/S-0215-behaviours.md) | Behaviours (standing orders) and their processing | simulation | 10 | approved | — |
| [S-0102](rendering/S-0102-draw-order.md) | Draw order, render layers, depth compositing, sprite placement and shadow pixels | rendering | 3 | approved | — |
| [S-0104](rendering/S-0104-lighting.md) | Lighting | rendering | 9 | approved | — |
| [S-0105](rendering/S-0105-voxel-rendering.md) | Rendering of voxel models | rendering | 5 | approved | — |
| [S-0301](combat/S-0301-firing-cycle.md) | Firing cycle (delay between shots, bursts, ammunition and reloading, the firing check) | combat | 1 | approved | — |
| [S-0302](combat/S-0302-damage-resolution.md) | Damage resolution (shot damage, defences, versus values, rounding, healing, health thresholds) | combat | 8 | approved | — |
| [S-0303](combat/S-0303-area-damage.md) | Area damage (spread, falloff, cells scanned, walls, bridges and detonation effects) | combat | 2 | approved | — |
| [S-0304](combat/S-0304-special-versus-values.md) | Special versus values (what zero and small versus values mean for firing, retaliation and automatic targeting) | combat | 2 | approved | — |
| [S-0305](combat/S-0305-armour-classes.md) | Armour classes (the class list, its order, and how a type's armour value is read) | combat | 1 | approved | — |
| [S-0307](combat/S-0307-range.md) | Weapon range test | combat | 1 | approved | — |
| [S-0308](combat/S-0308-weapon-choice.md) | Weapon choice (primary or secondary per target, elite weapons, multi-weapon types, deployed, passenger and garrison weapons) | combat | 1 | approved | — |
| [S-0310](combat/S-0310-turrets-facing-for-fire.md) | Turrets and facing for fire (aiming tolerance, which facing aims, omni-fire, firing while moving) | combat | 2 | approved | — |
| [S-0311](combat/S-0311-veterancy.md) | Veterancy (experience from kills, ranks, abilities, elite weapons, starting experience) | combat | 1 | approved | — |
| [S-0312](combat/S-0312-death-and-destruction.md) | Death and destruction (death weapons, debris, passengers, crews, survivors, sinking, crashing and building remains) | combat | 2 | approved | — |
| [S-0313](combat/S-0313-firing-positions.md) | Firing positions (where a shot starts, burst alternation, garrison muzzles, spray attack) | combat | 1 | approved | — |
| [S-0314](combat/S-0314-healing-repair.md) | Healing and repair in combat (healers, self-healing, house-wide healing, repair depot, hospital, building repair) | combat | 3 | approved | — |
| [S-0006](rules-import/S-0006-rules-layering.md) | Rules layering | rules-import | 7 | approved | — |
| [S-0009](formats/S-0009-map-file-semantics.md) | Map file semantics (geometry, terrain streams, waypoint markers, start positions, preview) | formats | 10 | approved | — |
| [S-0010](formats/S-0010-theater-tileset-semantics.md) | Theater data, tilesets and the terrain values of a cell | formats | 8 | approved | — |
| [S-0112](rendering/S-0112-overlay-terrain-object-drawing.md) | Overlay and terrain object drawing | rendering | 5 | approved | — |
| [S-0202](simulation/S-0202-random-streams.md) | Random streams (generator, seeding, ranged draws, which systems draw from which stream) | simulation | 7 | approved | — |
| [S-0203](simulation/S-0203-tick-order.md) | Tick order | simulation | 8 | approved | — |
| [S-0207](movement/S-0207-ground-vehicle-movement.md) | Ground vehicle movement (drive locomotor) | movement | 5 | approved | — |
| [S-0208](infantry/S-0208-infantry-movement.md) | Infantry movement (standing positions, walking step and speed, crawling, scatter destinations) | infantry | 12 | approved | — |
| [S-0209](movement/S-0209-occupancy-crowding.md) | Cell occupancy and crowding (capacity, entry outcomes, waiting, re-routing, scatter requests) | movement | 14 | approved | — |
