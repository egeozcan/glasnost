# The map's special flags: claim ledger (M8 map flags)

Brief: `.claude/worktrees/M8-MAPFLAGS-BRIEF.md`. Seam sweep 1 ([seams-1.md](seams-1.md), row 23) found that the
map's special-flags section was never read. This step reads `Inert` and `HarvesterImmune` from the map's
`[SpecialFlags]` section (absent: no), keeps them in the world (`World::map_flags`, saved and hashed) and
consults them at every claim below. Save format 75 to 76, AI game format 36 to 37.

| Spec and claim | State | Where and test |
|---|---|---|
| S-0303 Import mapping, `Inert` and `HarvesterImmune` (booleans, absent: no) | tested | `convert/src/map/mod.rs` (`basic`), `special_flags_inert_and_harvester_immune`; world at load: `map_flags_load_save_and_hash` (also the save round trip and the hash) |
| S-0303 Import mapping, `DestroyableBridges` | not read, by the spec | A skirmish ignores it and uses the setup screen's option (S-0303 claim 18, S-0511 claim 1; `econ::setup::bridge_destruction` already says so). It stays in the importer's `NAMED` list for S-0303 (not interpreted, by the spec's own rule), now the only map key there. |
| S-0303 claim 2 (inert map: the area damage of every detonation does nothing, result missed) | tested | `combat::area::detonate`; `claim_2_inert_map_area_damage_does_nothing` (scenario A1 variant e: no hit, wall stage unchanged, exploding overlay stays, no draw; control without the flag hits) |
| S-0303 claim 2 (the flag does not suppress animation or debris) and claim 22 (debris on an inert map; A15 repeat a) | tested | `claim_2_and_22_inert_map_keeps_the_debris`: same pieces and same draws with and without the flag. The flag is only read in `detonate`, which the projectile's animation and debris do not go through. |
| S-0303 claim 9 (a vehicle of a game-wide harvester type is left out while the flag is set) | tested | `combat::area::harvester_immune`; `claim_9_harvester_immunity` (scenario A6) |
| S-0303 claim 9 with `quirks.flying_candidates_unfiltered` off (the listed-harvester exclusion applies to tracked flying candidates; on, they are not filtered) | tested | `flying_listed_harvester_and_immunity` (a hovering jumpjet listed as harvester) |
| S-0302 claim 6 and F3 case (ii) (m0 is 0 in an inert game for a hit that does not ignore defences) | tested | `combat::damage::modified`; `d9_inert_game` |
| S-0302 claim 7 (inert game: an undemolishable building still loses exactly 1 from a non-detonation hit) and D9 (a), (b) | tested | `d9_inert_game` (the sonic wave is stood in for by a direct hit, as D34 does; no wave is built here) |
| S-0302 Formulas, berserk duration (T is 0 in an inert game) and S-0610 P3 | tested | the berserk path calls `modified`; `p3_berserk_hit_in_an_inert_game_has_zero_duration` (with `quirks.berserk_zero_duration` on: dropped target, hunt queued, state ends at the next countdown update) |
| S-0309 table C, row "harvester immunity" (A14 part i) | tested | `combat::acquire` (`Verdict::Reject`); `table_c_harvester_immunity` |
| S-0318 claim 1 and SH1 part s run 2 (an inert map still bursts; the 6 ranged draws are taken; no object loses health, the fragments' own area damage doing nothing) | tested | `sh1s_run_2_inert_map_still_bursts_and_nothing_loses_health` (control without the flag: T and the fragments hit) |
| S-0315 Terms (delivered amount taken before "an inert map reduces it") | by construction | The delivered amount is S-0302's d0; the reduction is `modified`. No code of S-0315 reads the flag itself. |
| S-0401 (`[SpecialFlags]` `TiberiumGrows`, `TiberiumSpreads`) and S-0409 (`FogOfWar`) | not in this step | Both specs say they are ignored in a skirmish. The existing growth switch reads `[Basic] TiberiumGrowthEnabled` per S-0401 claim 4. |
| S-0511 (`DestroyableBridges`) | no change | See the second row. |

No request was needed: the specs state every rule used. Requests Q-1851 to Q-1854 are unused.

The harvester list is `[General] HarvesterUnit` as imported for S-0303 (`combat.general.harvester_units`, vehicle
type indices); an empty list makes the flag a no-op.
