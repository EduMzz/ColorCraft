# ColorCraft — application submission brief

**Status:** preparation document, not a submitted RFC or an acceptance announcement. Reviewed 2026-10-19. The initial proposal concerns ColorCraft only; the VectorCraft integration remains a separate PoC.

## Name, identity and project lead

- Display name: **ColorCraft**.
- Machine/crate/binary stem: `colorcraft`.
- Current repository: [EduMzz/ColorCraft](https://github.com/EduMzz/ColorCraft).
- Current project lead: Eduardo, [@EduMzz](https://github.com/EduMzz).
- A suite-owned namespace such as `ai.storyteller.colorcraft`, organization transfer and long-term maintenance responsibilities require agreement. They are not claimed by this independent preview.

## What it is

A standalone Rust desktop application for parametric palette engineering: colors, mathematical relationships, constraints, source captures and semantic intent in one editable project. It publishes resolved palettes other local applications can consume.

The two main objectives are a useful independent palette-engineering tool and dynamic palettes reusable by other programs. Full ArtCraft integration is an important objective within the latter, subject to team decisions. The [README](../README.md) provides product examples; the [roadmap](../ROADMAP.md) describes planned formulas, community research and later integration/cloud work.

## Why a separate application

A drawing application's swatches answer “what color should this object use?” ColorCraft focuses on “how should this family of colors behave when something changes?” Its document is the palette system itself: relationships, protected anchors, captures, semantic assignments and a stable publication identity.

Keeping this work independent allows a palette to serve several projects and applications without requiring one artwork document or editor to own it. Existing client swatch panels remain useful and retain their native editing conventions. ColorCraft is not a replacement for image editing, vector illustration, grading or a general asset library.

Review should consider overlap with present and planned suite features. The argument for independence is reusable parametric engineering across workflows, rather than duplicating each editor's picker.

## First useful release and current implementation

The 0.1.14 Windows x64 preview already includes local project editing, channel links, harmonies/ramps, minimum-contrast constraints, conflict explanations, capture, native color/profile data, semantic metadata, autosave/recovery, exports and local snapshot publication. It runs without the VectorCraft PoC, an account or a cloud service.

Scope excluded from the initial production claim: arbitrary user formulas, integrated clients in official applications, semantic-role resolution in clients, a public cloud service, multi-user/multi-device editing, other native packages, a browser application, signed installers and Launcher distribution.

## Shared code and architecture

The workspace has `crates/core`, `crates/storage`, `crates/desktop` and `crates/bridge`. It uses native egui/eframe and pinned `craft-ui`, `craft-cms`, `craft-settings` and `craft-atomic` from `craft-libs` revision `82a24b006eefa036b43c09a67fb9a3fbd17ffe63`.

No new shared suite crate is requested as a prerequisite. The bridge contract is mirrored in the experimental client, not yet an adopted `craft-libs` API. Ownership and any extraction need separate review. The docking adapter and deviations are recorded in [DOCKING-ADAPTER.md](DOCKING-ADAPTER.md). General CLI/MCP command exposure is not delivered yet and should be evaluated with the team.

## Provenance, licensing and review

The application was developed from Eduardo's Colorator palette workflow. Original Rust code is MIT OR Apache-2.0, with third-party notices retained. Numerical color work follows public color/ICC/WCAG references; the desktop uses documented open-source renderers and libraries. No proprietary editor binaries, code, presets or ICC bundles are included. Capture examples use supplied project artwork or synthetic documents.

This is an implementation provenance statement, not an independent legal or security audit. Review [NOTICE.md](../NOTICE.md), [ASSETS.md](../ASSETS.md), the pinned lockfile, package notices and the docking adapter. Supplied illustrations have documented application-use permission but no declared general open-asset license. Those rights must be resolved before a transfer or distribution that requires broader rights. Do not silently treat the code license as an image license.

## Validation and remaining adoption work

| Area | Evidence / current limit |
|---|---|
| Standalone checks | The 0.1.14 validation record reports formatting, warnings-denied Clippy and 107 workspace tests passing on Windows. Re-run on the submitted commit; see [development](DEVELOPMENT.md). |
| Product illustrations | Three native screenshots from the built application, using synthetic/demo projects; the Studio example is included. |
| Integration | Local native tests cover navigation, explicit revisions, sending, restart/replay and fallback. Client PoC checks have their own limitations; see [bridge validation](BRIDGE-VALIDATION.md). |
| Portability | Core WebAssembly checking is supported; full browser and other native-platform delivery remain work. |
| Distribution | Portable Windows evaluation package with notices and hashes. Official signing, installer conventions and Launcher entry are pending. |
| Governance | Project fit, review capacity, maintainer commitment, asset rights, repository/identity transfer and release ownership need agreement. |
| Accessibility and localization | Existing keyboard/accessibility and PT/EN UI work needs broader review, including residual strings, scaling and assistive-technology behavior. |

The repository includes a Windows CI workflow for this workspace. It is not advertised as the suite's full release matrix. A green local test run does not replace maintainer code review or imply admission to the Launcher.

## Proposed adoption sequence

1. Review the product case, overlap and a named maintenance arrangement.
2. Follow the current `craft-contrib` RFC process for adding an application. Assign the RFC number at submission, rather than inventing one here.
3. Review the existing code, rights, dependencies, security boundaries and compatibility gaps. Agree which gaps block adoption and which block release/catalog entry.
4. If accepted, agree repository transfer, identity, shared guidelines, CI/release workflows and preservation of existing users' projects/preferences/releases. No transfer is authorized by this document alone.
5. Propose client integrations independently. Use the VectorCraft PoC as evidence, not as code automatically included in the ColorCraft submission.

## Sources and their status

- **Adopted process:** [craft-contrib RFC-0000](https://github.com/storytold/craft-contrib/blob/main/rfcs/0000-decision-process.md). Application additions use an RFC with a maintainer-started seven-day comment period; owners and maintainers decide the outcome.
- **Preparation reference, not adopted policy:** [RFC-0002 proposal / PR #3](https://github.com/storytold/craft-contrib/pull/3), checked through the GitHub API on 2026-10-19: closed, not merged. Its discussion separates ideas, existing-code submissions, adoption and release readiness. The topics above are a practical preparation checklist, not a claim that this draft is binding policy.
- **Existing distribution conventions:** [ArtCraft Launcher](https://github.com/storytold/craft-launcher#where-apps-are-installed) documents platform packages and checksum verification. ColorCraft does not yet claim those installers or catalog membership.

The referenced `storytold/craftrules` repository was not publicly accessible during preparation. Recheck any team-provided standards and changes before submission; do not claim full compliance with documents that could not be reviewed.
