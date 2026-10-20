# ColorCraft roadmap

**Stage:** functional Windows x64 preview, 0.1.14. **Reviewed:** 2026-10-19.

This roadmap describes product direction, not promised dates or approved ArtCraft work. Suite adoption, integration contracts, ownership and cloud infrastructure require later decisions by the ArtCraft team.

## Objectives

1. Be a standalone application for engineering color palettes through explicit relationships, constraints and editable intent.
2. Generate and dynamically edit palettes that other applications on the computer can use.
3. Within the second objective, pursue full integration with ArtCraft applications through separately reviewed client integrations.

The standalone workflow must remain useful without any integrated client, account or cloud service.

## Available in the preview

- Local projects, folders, cards/tree, selection, metadata, semantic roles, modes and undo/redo.
- RGB/HSV channel relationships, harmony recipes, discrete gradients and opaque sRGB minimum-contrast constraints.
- Full locks, direct-edit protection and navigable conflict diagnostics.
- Editable image/PDF/SVG captures, neighborhood sampling, painted masks and source links.
- Native color components and ICC data, explicit conversion choices and separate display previews.
- Autosave/recovery, local published snapshots and multiple export formats.
- A local v1 bridge receiver and exact project/color URI navigation; a separate VectorCraft client PoC provides integration evidence.

The [validation record](docs/BRIDGE-VALIDATION.md) distinguishes standalone checks from client-PoC checks. An evaluated Windows build does not establish support on every platform.

## User-defined parametric relations

Extend the current built-in channel rules with a granular relation editor for **two or more colors**, named variables and formulas. Users should be able to define which components participate, what remains independent, and how derived values respond to changes.

Illustrative future relationships:

```text
hue(accent) = wrap(hue(primary) + hue_offset)
saturation(accent) = clamp(saturation(primary) * saturation_scale, 0, 1)
lightness(midpoint) = (1 - t) * lightness(start) + t * lightness(end)
```

These are design examples, **not syntax accepted by the current application**. Color models, units, interpolation and function behavior must be explicit; “lightness” cannot silently mean HSV V, HSL L and OKLCH L interchangeably.

Planned design work:

- Named variables with meaningful labels, units, ranges and defaults.
- Explicit inputs, outputs and degrees of freedom for multi-color relationships.
- A clear distinction between directional formulas and reversible relationships; a formula is not automatically invertible.
- Bounds, gamut policy, locks and existing constraints remain respected. Conflicting equations, cycles and undefined math need useful diagnostics and atomic rollback.
- A bounded math expression language, without arbitrary script, file or network execution.
- Inspectable previews, examples, reusable definitions, deterministic persistence and undo/redo.
- An extension model developers can add to without duplicating solver or UI plumbing.

Start with community-validated examples and a small expression model. Do not add a general programming language before its interaction and numerical behavior are understandable.

## Community workflows and product research

Collect real use cases before deciding which controls and formulas deserve prominence:

- Brand revision and maintaining families of related accents.
- Text/surface palettes, light/dark assignments and contrast requirements.
- Illustration, game art, tonal ramps and reference-based palette development.
- Design tokens, developer handoff and revisions across creative applications.
- Print-oriented work with real source/output profiles and explicit color-management limits.

Ask contributors for the starting palette, the change they need, the relationships that should survive, the output application and the expected result. Favor reproducible, redistributable examples. Evaluate confusing failures, discoverability, keyboard access, language coverage and the distinction between numerical correctness and useful artistic behavior.

Community research is a product-development activity, not a claim of completed user studies. Use the [workflow template](.github/ISSUE_TEMPLATE/workflow.yml) to submit an example voluntarily.

## ArtCraft integration — subject to team decisions

The initial adoption proposal is **ColorCraft only**. The [VectorCraft fork](https://github.com/EduMzz/vectorcraft-colorcraft) remains a PoC, not an upstream integration submission.

Ideas to review with the team:

- Shared local palette discovery with stable identities and clear revision semantics.
- Client panels consistent with each application's native swatch workflow.
- Linked colors with local fallback values, explicit update acceptance and predictable unlinking.
- Exact “Edit in ColorCraft” navigation and confirmed submission of new colors.
- Native color/profile fidelity and explicit approximation where conversion is unsupported.
- Later resolution by semantic role and mode, rather than only by color ID.
- Ownership of any shared bridge crate, schema evolution, permission boundaries and migration.

The current file queue and published library are experimental interfaces of this implementation. They must not be presented as an approved standard for the whole suite. Each future client needs its own design, tests and review.

## Cloud publication, sharing and multiple devices — subject to team decisions

Explore remote palette publication, shared access and continuity across devices only after the team decides how those capabilities fit the suite.

Questions to settle include account ownership, access control, private/public visibility, version history, offline operation, concurrent edits, conflict resolution, service ownership, storage costs, export and deletion. Decide separately whether a palette is published for reading or collaboratively editable.

Local palette engineering and file access must remain useful if a remote service is unavailable. **No cloud upload, account system, shared server or cross-device synchronization exists in the current preview.**

## Additional platform clients — subject to suite guidance and team decisions

The current product is a Windows x64 preview. Evaluate native **macOS and Linux clients for ColorCraft itself** as the next platform work, following the suite's platform guidance and release practices. ArtCraft applications demonstrate multi-platform delivery, but this roadmap does not assume that every application must support every suite target or that ColorCraft has been accepted into the suite.

Before committing to a platform, review its native file dialogs and project handling, color/profile behavior, PDFium runtime and licensing, local palette library and bridge paths, accessibility, packaging/signing, CI coverage, release support and ongoing maintainer capacity. Keep the shared core portable where practical, while validating the complete desktop application and its user-facing workflows on each delivered target.

Consider **FreeBSD or a browser client** separately only if they fit the suite's then-current roadmap and have an owner. A WebAssembly core check is not a browser application, and a browser client would need a deliberate design for local files and the local integration bridge. Do not announce support or publish a platform build until it has passed the relevant suite gates and has an agreed maintenance path.

## Adoption and release readiness

- Complete code, dependency, asset-rights and provenance review.
- Confirm maintenance responsibilities and fit alongside existing suite applications.
- Align accepted identity, repository conventions and shared-library usage with the team.
- Evaluate suite CI/release workflows, signing, conventional installers and Launcher catalog admission after acceptance. Adoption and release/catalog readiness are separate decisions.
- Evaluate and validate macOS and Linux clients for ColorCraft, then consider other suite targets only through the review above; distinguish a portable core from a delivered desktop/web application.
- Improve accessibility, complete language review and broaden numerical/interoperability regressions.

See [ARTCRAFT-PROPOSAL.md](docs/ARTCRAFT-PROPOSAL.md) for the application-submission brief, sources and remaining gaps.
