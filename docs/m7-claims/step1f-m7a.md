# M7a step 1f claim ledger (S-0202 revision 29, S-0010 revision 18, S-0109 revision 29, S-0203 revision 36)

This is part 6 of M7a step 1. Four approved revisions landed on main after 75350df (`git diff 75350df 3d0c63d -- spec/`). Every changed claim gets one class, as in [step1e-m7a.md](step1e-m7a.md):
- *satisfied*: already true, with the test that shows it;
- *changed*: code changed, with the test;
- *pointer*: the claim only points at another spec's claim;
- *not built*: with the reason (a claim for a part not built yet);
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

| Spec | Revision | Answers |
|---|---|---|
| S-0202 random streams | 29 (from 28) | none |
| S-0010 theater data and tilesets | 18 (from 17) | none |
| S-0109 animation playback | 29 (from 28) | none |
| S-0203 tick order | 36 (from 35) | none |

All four revisions restate in their own text what S-0613 revision 5 (step 1e) already says: the particle's initial lifetime (F9), the loading-time particle request that meets no shared gas system, and the shared system's place at game start. No behaviour changed in our engine. Two tests were added to show claims that had no test of their own.

Requests: `spec/requests/` is unchanged in 75350df..3d0c63d. Every request in the four headers (S-0202: Q-0018, Q-0046, Q-0101, Q-0182, Q-0217, Q-0327, Q-0711; S-0010: Q-0009, Q-0018, Q-0041, Q-0046, Q-0055; S-0109: Q-0046, Q-0418, Q-0770, Q-0771; S-0203: Q-0018, Q-0412, Q-0430) was already `answered` before this step. No open request names one of these specs in `answered-by`, so no interim is retired. Q-1243 (gas systems of other kinds) stays open. New requests: none (block Q-1152 to Q-1159 unused).

M7a code: the gas code (`glasnost_sim::gas`) is where these claims land (see the rows). Mind control (`mind`), gattling (`gattling`), psychic weapons (`psychic`) and the infantry absorber (`structures::absorb`) do not depend on the game-start update order, on animation middle events or on particle creation draws, so no change touches them.

Save format: unchanged at 50. AI game format: unchanged at 19. Hashes: unchanged (no simulation code changed; only a citation in `gas.rs`).

## S-0202 revision 29

| Claim | Class | Notes |
|---|---|---|
| Header (`revision` 29, `approved-on`) | (c) | |
| 15, particle table, *Every particle*: with a nonzero modulus the variation lies in 0 … \|modulus\| − 1; with a zero lifetime field the zero-divisor case applies; the variation is added to the lifetime field to give the initial lifetime in ticks (S-0613 F9) | satisfied | `gas::creation_draws` (its citation now also names S-0202 claim 15 and the range). `f9_initial_lifetime` (fields 1, 0, 40 000, −70 000; the zero field takes the zero-divisor case), `claim_15_railgun_lifetime_variation` (new: a railgun particle of field 1000 made by a middle event gets 1000 + \|r rem 10\|, and the variation lies in 0 … 9). |

## S-0010 revision 18

| Claim | Class | Notes |
|---|---|---|
| Header (`revision` 18, `approved-on`) | (c) | |
| 19: a tile animation started while the map loads follows the creation-time rule: with μ = 0 (⌊N / 2⌋ of the sprite of S-0109 claim 25) its middle event happens at creation, not at its first update (S-0109 claim 14) | satisfied | `load::recalc_cells` creates the tile animation through `load::anim_created_at`, which runs the middle event at once for μ = 0. `claim_15_crater_before_the_data_stream` (a μ = 0 tile animation's crater acts during loading, before the overlay data stream), `edge_loading_tile_animation_request_makes_no_cloud` (new: a μ = 0 tile animation requesting 2 gas particles takes 4 creation draws while loading, 2 starts of 2 requests, and makes no cloud). |
| 19, Backing table, new row (the loading-time middle event of a μ = 0 tile animation) | pointer | S-0109 claim 14. |

## S-0109 revision 29

| Claim | Class | Notes |
|---|---|---|
| Header (`revision` 29, `approved-on`) | (c) | |
| 14: during map loading the shared gas system does not exist yet, so a loading-time μ = 0 request meets no shared system and the original stops; our engine's departure is in S-0613's Edge cases | satisfied | The `(_, _, None)` branch of `gas::middle_event_particle` takes the creation draws and makes no particle (step 1e). `edge_loading_request_makes_no_cloud` (a cell animation), `edge_loading_tile_animation_request_makes_no_cloud` (new, a tile animation). The importer warns for a cell animation (`loading_time_particle_request_warns`); theater tile animations are not seen by the rules import, the seam already named in [S-0613.md](S-0613.md). |
| Measurement scenario (claim 14's repeat with a positive particle count uses a play-time creator, not a map-placed animation) | (c) | `in_play_creation_takes_the_loading_composition` already uses a play-time creator for the particle case. |

## S-0203 revision 36

| Claim | Class | Notes |
|---|---|---|
| Header (`revision` 36) | (c) | |
| Terms, *Shared gas system* (new) | pointer | S-0613 claim 1. |
| 3: the last map-placed infantry unit is directly followed by the first map-placed building, unless a game-start animation or the shared gas system stands between them | satisfied | In our engine animations are not world objects (they have no playback clock in the simulation), and the shared system is created after the starting units and the game-start pass (`load::new_game`). So nothing stands between the two blocks. `s0203_update_order_of_map_objects` (the kinds in block order), `claim_1_shared_system_enters_play_last`. |
| 3, *Other game-start objects* (new): the game-start animations and the shared gas system are not map-placed; each enters play by appending at the point its game-start event falls in S-0412 claim 14's order; no observable consequence and no adjacency claimed | satisfied (shared system) / not built (animations) | Shared system: as S-0613 claim 1, `claim_1_shared_system_enters_play_last`. Animations: not built. Animations are not objects in the update order, the existing seam ("animations have no playback clock in the simulation", the *Animation start events* bullet of [development.md](../development.md)). The claim gives no observable consequence and low confidence. |
| 3, Backing (a new `unmeasured` line) and Confidence (low for the place of game-start animations and the shared system) | (c) | |
