# Measurements

Records of what the original game does when run and observed: timings, counts, credits, positions, health readings, win rates. The project lead (oracle operator) takes them from the stock game, by hand or with a scripted harness, and commits them. See [PROVENANCE.md](../PROVENANCE.md) sections 5 and 7.

## Rules

- **Stock game only.** Stock data, a hash-checked executable, every gameplay mod and AI change off. Record the settings.
- **Observables only.** No code, addresses, configuration or notes from the harness enter this repository.
- **Values that mirror stock rules data stay local.** If the measured value is the same thing as a rules value (a cost, a reload time, a range), the record commits the scenario, method and relation (`value-visibility: local`), and the value goes in the git-ignored `measurements/local/M-xxxx.json`. Values that are not rules data (tick rate, ordering, win rates under stated settings) are committed (`value-visibility: committed`).
- **The measurement wins** over a spec. A contradiction sends the spec back for revision.

## Layout

| Path | Contents |
|---|---|
| `<topic>/M-xxxx-*.md` | One record per file, same topics as `spec/` |
| `TEMPLATE.md` | The record template |
| `local/` | Git-ignored local-only values; never committed |

## Index

| ID | Title | Topic | Confirms | Visibility |
|---|---|---|---|---|
| — | *No measurements yet* | | | |
