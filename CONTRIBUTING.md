# Contributing to ColorCraft

ColorCraft is an independent Rust palette-engineering application being prepared for consideration by the ArtCraft team. Read [README](README.md), [ROADMAP](ROADMAP.md), [AGENTS](AGENTS.md) and [development](docs/DEVELOPMENT.md) first. The initial application proposal includes ColorCraft only; changes to other editors belong in separately reviewed integrations.

## Help shape the product

Describe a real workflow: the starting palette, the change you need, the relationships that should survive, your output application and the expected result. The workflow issue template is intended to collect these examples before generalizing the formula editor or client integrations. Use only examples you can share, and identify any sample-file license.

For bugs, include the ColorCraft version, OS, interface language/scale, reproduction steps and expected/actual results. A small synthetic project is often more useful than a large personal document. Report incomplete translations with the affected panel/control and language.

## Code and documentation

- Keep changes focused. Explain the product behavior and what was actually verified.
- Preserve project identities, native color/ICC data, undo/redo and atomic failure behavior.
- Express editable project changes through the existing command mechanism. Avoid mutation during UI rendering.
- Cover solver, persistence and protocol changes with meaningful regressions; run the checks in the development guide.
- Update both PT/EN interface text when controls change. User-provided names and metadata are not interface translations.
- Record every new asset's origin and terms in [ASSETS.md](ASSETS.md). Do not copy proprietary application code, icons, presets, artwork or ICC bundles.
- Original contributions use MIT OR Apache-2.0 unless explicitly documented otherwise. Existing third-party and supplied-artwork terms remain separate.
- Do not commit build output, local palettes, recovery files, credentials or machine-specific data.

The repository includes a PR template and Windows/core CI. No contribution here automatically submits a suite-admission RFC, transfers ownership or promises an official release. Those are decisions for the project lead and the ArtCraft team under the current contribution process.
