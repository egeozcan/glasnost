# Specs

Plain-language behaviour specs for the engine. Each spec passed the scrub pipeline in [PROVENANCE.md](../PROVENANCE.md) (sections 2–4) before it was committed here. Implementer sessions read specs; they never edit them, except to add requests under `requests/`.

## How a spec arrives

1. The spec agent drafts it in its own workspace, or answers a request from `requests/`.
2. Lint, then a fresh scrubber session, then sign-off by the project lead.
3. The lead commits it as `spec/<topic>/S-xxxx-short-name.md` with `spec: add S-xxxx <title>`.

A spec that contradicts a measurement goes back for revision; the measurement wins. Revisions bump `revision` and go through the same pipeline.

## Layout

| Path | Contents |
|---|---|
| `<topic>/S-xxxx-*.md` | One spec per file, grouped by game topic |
| `TEMPLATE.md` | The spec template |
| `glossary.md` | Our own vocabulary for game concepts (to be written with the first specs) |
| `requests/` | Spec requests (`Q-xxxx`) from implementer sessions |
| `incidents/` | Incident logs (`I-xxxx`), see PROVENANCE.md section 8 |

Topics: `simulation`, `combat`, `movement`, `production`, `economy`, `structures`, `infantry`, `vehicles`, `aircraft`, `naval`, `superweapons`, `special-abilities`, `visibility`, `rules-import`, `formats`, `rendering`, `triggers`, `ui`. Add a topic when a spec does not fit; never organise by the original program's modules.

## Index

| ID | Title | Topic | Rev | Status | Measurements |
|---|---|---|---|---|---|
| [S-0001](rules-import/S-0001-ini-lexical-rules.md) | INI lexical rules | rules-import | 3 | approved | — |
| [S-0002](rules-import/S-0002-ini-duplicates.md) | INI duplicates and name resolution | rules-import | 4 | approved | — |
| [S-0003](rules-import/S-0003-ini-value-forms.md) | INI value forms | rules-import | 3 | approved | — |
| [S-0004](rules-import/S-0004-type-registration-order.md) | Type registration order | rules-import | 3 | approved | — |
| [S-0005](rules-import/S-0005-data-file-precedence.md) | Data file precedence (loose files, archive mount order, name lookup) | rules-import | 3 | approved | — |
| [S-0006](rules-import/S-0006-rules-layering.md) | Rules layering | rules-import | 4 | approved | — |
| [S-0007](rules-import/S-0007-art-lookup.md) | Art lookup (image indirection, sprite or voxel, companion files, theater substitution) | rules-import | 2 | approved | — |
