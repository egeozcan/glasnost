# S-0216 r70 and M-0001 to M-0003 (Setup revisions): claim ledger, M8 step A3 intake 91

Requests filed: none (Q-2486 to Q-2495 unused). Quirks: none added or changed.

| claim part | status |
|---|---|
| S-0216 claim 8: the human player house is British (replaces r69's side-A rule) | Tested: `oracle::HUMAN_COUNTRY` and `oracle::human_country` look the country up in the loaded rules; `arena_setup::pt6_game_start_houses_in_claim_19s_order` (human is British, not side A's country) |
| No such country in the loaded data: clear error, no fall back | Tested: `pt6_rules_without_the_british_country_fail_with_a_clear_error` |
| Seed 1000 + s (r70 wording, M-0001 to M-0003) | No change: `game_seed` and `pt6_seed_number_s_seeds_the_stream_with_1000_plus_s` already match |
| Country picks the base unit (S-0009 claim 45); house draws independent (S-0202 claim 14) | No change; the human house's base unit is removed before the groups (claim 8, Removal) |
| Open question Q-2446 q1 removed | No change |

Intake 87's three choices: r70 and the record revisions settle none.
### M-0019 N0 (local lane, release, with the install)

| Seed number | 0 | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|---|
| M-0019 N0 | 706 | 709 | 711 | 718 | 706 | 701 |
| Ours (British human house) | 706 | 709 | 711 | 718 | 706 | 701 |
| Gap | 0 | 0 | 0 | 0 | 0 | 0 |

### Arena `--record all --threads 4`, 21ad0fe against this branch

The two runs print the same text on all 198 lines except the "played ... in N s" timings. Every pairing has the same win counts per side, so none of the 13 failing pairings changed (ROBO-HTK, TELE-APOC, TELE-V3, TELE-ROBO, TELE-BFRT, CAOS-E1, DISK-FV, INIT-E2, MIND-E1, CAOS-APOC, CAOS-TTNK, CAOS-E2, YURI-SHK) and no other pairing flipped. Failing: 10 of 96, 2 of 24, 1 of 35. Expected: the country only picks the base unit, which is removed before the groups, and the house-creation draws do not depend on it.

Change stays inside `crates/tools` (diff stat: oracle.rs, arena_setup.rs, this file); sweeps, `glasnost-ai` and the AI quick lane skipped. No hashed state, format or golden change.

Checks: clippy `--workspace --all-targets -D warnings` clean; `cargo fmt --check` clean (after one `cargo fmt`); `cargo xtask check`: stock-guard 99 known problems, canary ok; `cargo test --workspace --no-fail-fast`: no failures.
