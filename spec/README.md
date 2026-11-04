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
| [S-0304](combat/S-0304-special-versus-values.md) | Special versus values (what zero and small versus values mean for firing, retaliation and automatic targeting) | combat | 6 | approved | — |
| [S-0305](combat/S-0305-armour-classes.md) | Armour classes (the class list, its order, and how a type's armour value is read) | combat | 1 | approved | — |
| [S-0403](production/S-0403-build-time.md) | Build time (progress steps, base time from cost, country and type factors, low power, several factories, walls) | production | 3 | approved | — |
| [S-0201](simulation/S-0201-game-speed-tick-rate.md) | Game speed and tick rate | simulation | 31 | approved | — |
| [S-0510](infantry/S-0510-spies-infiltration.md) | Spies and infiltration | infantry | 3 | approved | — |
| [S-0310](combat/S-0310-turrets-facing-for-fire.md) | Turrets and facing for fire (aiming tolerance, which facing aims, omni-fire, firing while moving) | combat | 8 | approved | — |
| [S-0410](ui/S-0410-radar-minimap.md) | Radar and minimap (when radar is available, what the radar map shows and how it is coloured) | ui | 20 | approved | — |
| [S-0211](movement/S-0211-land-types-passability.md) | Land types and passability | movement | 18 | approved | — |
| [S-0112](rendering/S-0112-overlay-terrain-object-drawing.md) | Overlay and terrain object drawing | rendering | 13 | approved | — |
| [S-0524](triggers/S-0524-mp-trigger-completion.md) | Multiplayer trigger completion (remaining stock actions and entry raises) | triggers | 9 | approved | — |
| [S-0601](combat/S-0601-mind-control.md) | Mind control | combat | 21 | approved | — |
| [S-0003](rules-import/S-0003-ini-value-forms.md) | INI value forms | rules-import | 19 | approved | — |
| [S-0005](rules-import/S-0005-data-file-precedence.md) | Data file precedence (loose files, archive mount order, name lookup) | rules-import | 9 | approved | — |
| [S-0002](rules-import/S-0002-ini-duplicates.md) | INI duplicates and name resolution | rules-import | 8 | approved | — |
| [S-0004](rules-import/S-0004-type-registration-order.md) | Type registration order | rules-import | 20 | approved | — |
| [S-0009](formats/S-0009-map-file-semantics.md) | Map file semantics (geometry, terrain streams, waypoint markers, start positions, preview) | formats | 44 | approved | — |
| [S-0001](rules-import/S-0001-ini-lexical-rules.md) | INI lexical rules | rules-import | 7 | approved | — |
| [S-0012](formats/S-0012-map-block-encodings.md) | Map block encodings | formats | 5 | approved | — |
| [S-0014](formats/S-0014-game-font-layout.md) | Game bitmap font file layout | formats | 4 | approved | — |
| [S-0006](rules-import/S-0006-rules-layering.md) | Rules layering | rules-import | 21 | approved | — |
| [S-0013](formats/S-0013-voxel-lighting-table-layout.md) | voxel lighting table file layout | formats | 3 | approved | — |
| [S-0101](rendering/S-0101-isometric-projection.md) | Isometric projection, height lift, view position and scroll limits | rendering | 13 | approved | — |
| [S-0011](triggers/S-0011-map-triggers-mp.md) | Map triggers (multiplayer subset) | triggers | 26 | approved | — |
| [S-0010](formats/S-0010-theater-tileset-semantics.md) | Theater data, tilesets and the terrain values of a cell | formats | 19 | approved | — |
| [S-0202](simulation/S-0202-random-streams.md) | Random streams (generator, seeding, ranged draws, which systems draw from which stream) | simulation | 62 | approved | — |
| [S-0204](simulation/S-0204-coordinates-distance-facing.md) | Coordinates, distance and facing | simulation | 10 | approved | — |
| [S-0210](movement/S-0210-crushing.md) | Crushing (who crushes whom, when, the outcome, walls, warnings, slowdown and automatic crushing) | movement | 31 | approved | — |
| [S-0307](combat/S-0307-range.md) | Weapon range test | combat | 21 | approved | — |
| [S-0315](combat/S-0315-suppression-panic.md) | Suppression and panic (infantry fear, prone and panic in combat; parasite suppression) | combat | 9 | approved | — |
| [S-0313](combat/S-0313-firing-positions.md) | Firing positions (where a shot starts, burst alternation, garrison muzzles, spray attack, shot animation position) | combat | 42 | approved | — |
| [S-0311](combat/S-0311-veterancy.md) | Veterancy (experience from kills, ranks, abilities, elite weapons, starting experience) | combat | 13 | approved | — |
| [S-0401](economy/S-0401-ore-and-gems.md) | Ore and gems (resource cells, bales and value, growth, spread, ore mines) | economy | 17 | approved | — |
| [S-0406](structures/S-0406-building-placement.md) | Building placement (foundation cells, the cell test, adjacency, building off allies, shroud, walls and gates, upgrades) | structures | 11 | approved | — |
| [S-0412](simulation/S-0412-skirmish-setup-rules.md) | Skirmish setup rules (setup values, starting credits, random lobby choices, handicap, alliances at game start, superweapon and crate settings) | simulation | 14 | approved | — |
| [S-0413](simulation/S-0413-country-bonuses.md) | Country bonuses (what each country field does, which spec owns it, and the fields with no effect) | simulation | 6 | approved | — |
| [S-0414](simulation/S-0414-victory-and-defeat.md) | Victory and defeat (the defeat tests of the normal and short game, what defeat does, when the game ends, the end delay, surrender and alliance orders) | simulation | 30 | approved | — |
| [S-0416](ui/S-0416-credit-display.md) | Credit display (the ticking credits counter on the sidebar) | ui | 3 | approved | — |
| [S-0503](movement/S-0503-hover-amphibious.md) | Hover and amphibious movement | movement | 28 | approved | — |
| [S-0508](structures/S-0508-garrisons.md) | Garrisons (occupying buildings, ownership of civilian garrisons, evacuation and clearing) | structures | 9 | approved | — |
| [S-0509](structures/S-0509-engineers-capture.md) | Engineers and capture | structures | 7 | approved | — |
| [S-0512](superweapons/S-0512-superweapon-charge.md) | "Superweapons: charge and readiness" | superweapons | 21 | approved | — |
| [S-0515](structures/S-0515-tech-buildings.md) | Tech buildings (manned state, acquisition bonus, periodic income, ownership changes) | structures | 5 | approved | — |
| [S-0525](combat/S-0525-radiation-fields.md) | Radiation fields | combat | 4 | approved | — |
| [S-0520](economy/S-0520-crates.md) | Crates (placement, replacement, pickup and power-up effects) | economy | 6 | approved | — |
| [S-0609](combat/S-0609-gattling-spin-up.md) | Gattling spin-up | combat | 6 | approved | — |
| [S-0612](superweapons/S-0612-psychic-dominator.md) | Psychic Dominator superweapon | superweapons | 4 | approved | — |
| [S-0614](combat/S-0614-parasites.md) | Parasites | combat | 21 | approved | — |
| [S-0110](rendering/S-0110-shroud-rendering.md) | Shroud and fog rendering | rendering | 12 | approved | — |
| [S-0701](ui/S-0701-sound-selection.md) | Sound selection | ui | 36 | approved | — |
| [S-0213](simulation/S-0213-human-orders.md) | Human orders (the order set, when orders apply, and what each order changes) | simulation | 96 | approved | — |
| [S-0511](structures/S-0511-bridges.md) | Bridges (destruction, collapse, falling and repair) | structures | 8 | approved | — |
| [S-0407](production/S-0407-tech-tree.md) | Tech tree (what a house may build: prerequisites and generic groups, tech level, country lists, stolen tech, build limits, captured buildings, secret labs) | production | 29 | approved | — |
| [S-0417](structures/S-0417-factory-unloading.md) | Factory unloading (doors, exit movement and rally destinations) | structures | 68 | approved | — |
| [S-0411](ui/S-0411-sidebar-behaviour.md) | Sidebar behaviour (build icons, clicks and the orders they give, icon states, queue counts, ready and placement) | ui | 49 | approved | — |
| [S-0408](economy/S-0408-sell-repair.md) | Selling and building repair (refund, what a sale releases, selling vehicles and walls, which buildings can be sold or repaired) | economy | 22 | approved | — |
| [S-0106](rendering/S-0106-voxel-tilt-slopes.md) | Slope shapes, terrain surface height and voxel tilt on slopes | rendering | 12 | approved | — |
| [S-0522](movement/S-0522-paradrops.md) | Parachutes and drops | movement | 14 | approved | — |
| [S-0404](production/S-0404-production-queue-payment.md) | Production queue and payment (production lines, price, pay-as-you-go steps, waiting for money, hold, cancel and refunds, queue limit, completion) | production | 23 | approved | — |
| [S-0415](structures/S-0415-mcv-deploy-undeploy.md) | Deploying a vehicle into a building and packing it up again (construction yard vehicles and other deploying vehicles) | structures | 18 | approved | — |
| [S-0517](combat/S-0517-deployable-units.md) | Deployable units (in-place vehicle deploy cycle, deploy-fire vehicles, deployed infantry firing) | combat | 10 | approved | — |
| [S-0206](movement/S-0206-turn-rates.md) | Turn rates and turning | movement | 16 | approved | — |
| [S-0207](movement/S-0207-ground-vehicle-movement.md) | Ground vehicle movement (drive locomotor) | movement | 83 | approved | — |
| [S-0209](movement/S-0209-occupancy-crowding.md) | Cell occupancy and crowding (capacity, entry outcomes, waiting, re-routing, scatter requests) | movement | 55 | approved | — |
| [S-0302](combat/S-0302-damage-resolution.md) | Damage resolution (shot damage, defences, versus values, rounding, healing, health thresholds) | combat | 40 | approved | — |
| [S-0312](combat/S-0312-death-and-destruction.md) | Death and destruction (death weapons, debris, passengers, crews, survivors, sinking, crashing and building remains) | combat | 51 | approved | — |
| [S-0205](simulation/S-0205-number-domains.md) | Number domains (integers, floating point, the round-toward-zero mode, conversions) | simulation | 10 | approved | — |
| [S-0203](simulation/S-0203-tick-order.md) | Tick order | simulation | 43 | approved | — |
| [S-0409](visibility/S-0409-shroud-sight.md) | Shroud and sight (explored and seen cells, sight radius and shape, height blocking, gap generators, re-covering, reveal on fire) | visibility | 16 | approved | — |
| [S-0605](structures/S-0605-infantry-absorber.md) | Infantry absorber (who may enter an absorbing building, entry, holding, release) | structures | 9 | approved | — |
| [S-0603](structures/S-0603-grinder.md) | Grinder (recycling units into credits) | structures | 9 | approved | — |
| [S-0606](structures/S-0606-vehicle-bunker.md) | Vehicle bunker | structures | 8 | approved | — |
| [S-0507](combat/S-0507-transports.md) | Transports (eligibility, capacity, boarding, passenger order, unloading, open-topped passengers) | combat | 9 | approved | — |
| [S-0214](ui/S-0214-order-targeting-rules.md) | Order targeting rules (what a click means for each kind of object, and group orders) | ui | 78 | approved | — |
| [S-0212](movement/S-0212-pathfinding.md) | Pathfinding (observable behaviour) | movement | 87 | approved | — |
| [S-0308](combat/S-0308-weapon-choice.md) | Weapon choice (primary or secondary per target, elite weapons, multi-weapon types, deployed, passenger and garrison weapons) | combat | 17 | approved | — |
| [S-0303](combat/S-0303-area-damage.md) | Area damage (spread, falloff, cells scanned, walls, bridges and detonation effects) | combat | 32 | approved | — |
| [S-0215](simulation/S-0215-behaviours.md) | Behaviours (standing orders) and their processing | simulation | 104 | approved | — |
| [S-0502](movement/S-0502-jumpjet-movement.md) | Movement of jumpjets | movement | 21 | approved | — |
| [S-0309](combat/S-0309-target-acquisition.md) | Target acquisition (automatic searches, candidate tests, threat score, retaliation, target switching) | combat | 95 | approved | — |
| [S-0208](infantry/S-0208-infantry-movement.md) | Infantry movement (standing positions, walking step and speed, crawling, scatter destinations) | infantry | 34 | approved | — |
| [S-0405](economy/S-0405-power.md) | Power (output and drain, damaged power plants, low power, blackouts, which buildings stop) | economy | 51 | approved | — |
| [S-0608](combat/S-0608-drain.md) | Drain (attaching to a building, draining its power and credits, and letting go) | combat | 6 | approved | — |
| [S-0607](combat/S-0607-magnetic-lift.md) | Magnetic lift (seizing a vehicle's locomotor, lifting, dropping and the fall) | combat | 38 | approved | — |
| [S-0518](combat/S-0518-linked-weapons.md) | Linked weapons (prism support beams, charging of overpowerable buildings, beam drawing) | combat | 6 | approved | — |
| [S-0519](combat/S-0519-terror-attack-animals.md) | Terror and attack animals | combat | 18 | approved | — |
| [S-0616](combat/S-0616-particle-systems.md) | Particle systems (smoke, fire, spark, railgun) | combat | 27 | approved | — |
| [S-0611](combat/S-0611-mutation.md) | Mutation | combat | 4 | approved | — |
| [S-0610](combat/S-0610-psychic-weapons.md) | Psychic weapons | combat | 12 | approved | — |
| [S-0107](rendering/S-0107-infantry-sprite-sequences.md) | Infantry sprite sequences | rendering | 31 | approved | — |
| [S-0105](rendering/S-0105-voxel-rendering.md) | Rendering of voxel models | rendering | 39 | approved | — |
| [S-0007](rules-import/S-0007-art-lookup.md) | Art lookup (image indirection, sprite or voxel, companion files, theater substitution) | rules-import | 15 | approved | — |
| [S-0104](rendering/S-0104-lighting.md) | Lighting | rendering | 27 | approved | — |
| [S-0109](rendering/S-0109-animation-playback.md) | Animation playback | rendering | 42 | approved | — |
| [S-0111](rendering/S-0111-in-game-overlays.md) | In-game overlays (health bars, pips, selection marks, insignia) | rendering | 25 | approved | — |
| [S-0314](combat/S-0314-healing-repair.md) | Healing and repair in combat (healers, self-healing, house-wide healing, repair depot, hospital, building repair) | combat | 107 | approved | — |
| [S-0402](economy/S-0402-harvest-cycle.md) | Harvest cycle (loading, field and refinery choice, docking, unloading, income, slave and chrono miners) | economy | 80 | approved | — |
| [S-0501](movement/S-0501-aircraft.md) | Aircraft flight, attack runs and pads | movement | 103 | approved | — |
| [S-0504](movement/S-0504-naval-units.md) | Naval units (ship locomotion, wakes, sinking, underwater vessels, docking at yards) | movement | 26 | approved | — |
| [S-0108](rendering/S-0108-building-sprites.md) | Building sprites | rendering | 106 | approved | — |
| [S-0613](combat/S-0613-gas-clouds.md) | Gas clouds | combat | 13 | approved | — |
| [S-0513](superweapons/S-0513-superweapon-effects.md) | Superweapon effects | superweapons | 108 | approved | — |
| [S-0516](visibility/S-0516-cloak-stealth.md) | Cloak and stealth (cloaking, cloak fields, sensors, disguise) | visibility | 13 | approved | — |
| [S-0615](combat/S-0615-special-hit-effects.md) | Special-hit effects (death-kind animations, the burning walker and the disc laser) | combat | 4 | approved | — |
| [S-0301](combat/S-0301-firing-cycle.md) | Firing cycle (delay between shots, bursts, ammunition and reloading, the firing check) | combat | 69 | approved | — |
| [S-0103](rendering/S-0103-palettes-house-colour.md) | Palettes and house colour | rendering | 17 | approved | — |
| [S-0306](combat/S-0306-projectiles.md) | Projectiles (launch, flight, homing, scatter, detonation) | combat | 65 | approved | — |
| [S-0514](combat/S-0514-chrono-weapons.md) | Chrono weapons (temporal erasure) | combat | 16 | approved | — |
| [S-0102](rendering/S-0102-draw-order.md) | Draw order, render layers, depth compositing, sprite placement and shadow pixels | rendering | 41 | approved | — |
| [S-0506](combat/S-0506-missiles.md) | Missiles (spawned missiles, launch cycle and rocket flight) | combat | 20 | approved | — |
| [S-0316](combat/S-0316-electric-bolt-shots.md) | Electric-bolt shots (the shot, its sparks, and how the bolt is drawn) | combat | 35 | approved | — |
| [S-0703](ui/S-0703-music-playback.md) | Music playback (track eligibility, next-track order, shuffle and repeat) | ui | 58 | approved | — |
| [S-0318](combat/S-0318-shrapnel-fragments.md) | Shrapnel fragments (when a detonation bursts into further shots, how many, and where they go) | combat | 27 | approved | — |
| [S-0317](combat/S-0317-voxel-debris.md) | Debris pieces with voxel models (creation, flight, bounces, end of life, removal) | combat | 57 | approved | — |
| [S-0319](combat/S-0319-bouncing-animations.md) | Bouncing animations (sprite debris and meteors): motion, landing and end | combat | 26 | approved | — |
| [S-0216](simulation/S-0216-units-placed-mid-game.md) | Units placed after the game has started | simulation | 67 | approved | — |
| [S-0113](rendering/S-0113-body-rocking.md) | Body rocking | rendering | 17 | approved | — |
