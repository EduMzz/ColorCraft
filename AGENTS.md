# ColorCraft — instructions for contributors and coding agents

Read README.md, ROADMAP.md, CONTRIBUTING.md and docs/DEVELOPMENT.md before implementation. ColorCraft is a standalone Rust application being prepared for ArtCraft consideration, not an official suite application. docs/ARTCRAFT-PROPOSAL.md distinguishes current policy, preparation references and remaining gaps. Do not treat unmerged RFCs as adopted rules.

## Scope and architecture

- The initial submission is ColorCraft only. Keep the separate VectorCraft fork a PoC; do not silently include or propose its changes upstream.
- Core owns project data, solver, commands, diagnostics and export; storage owns persistence; desktop owns UI; bridge owns the local contract/queue. Keep UI dependencies out of the core.
- Project mutations use commands and atomic undo/redo. Viewing controls or navigating a preview must not change colors. Preserve stable IDs and typed diagnostic provenance.
- Preserve native components and ICC bytes separately from display previews. Never silently advertise unsupported conversion as faithful.
- Current channel rules are not a general formula editor. Mark planned formula, cloud and integration features clearly as planned.

## Safety and evidence

- No panics or unsafe code in shipped paths. Handle fallible input with results and bound files, allocations, recursion and numerical input. Tests may unwrap where appropriate.
- Never discard source data, user projects, old distributions, preferences or running user instances to complete a test/build.
- Use synthetic examples and isolated library/queue/data roots. Do not use personal palettes as public fixtures.
- Use public specifications and open-source dependencies. Do not run or inspect proprietary creative applications to reproduce their behavior, or copy their code/assets/profiles.
- Every asset needs documented provenance and rights. Do not reinterpret application-use permission for supplied illustrations as a general open license.
- Run the relevant checks in docs/DEVELOPMENT.md after code changes; record failures and untested cases accurately. Documentation-only changes need link, example and command review rather than redundant tests.
- Keep README and ROADMAP current. English is the public presentation language; maintain PT/EN UI catalogs and preserve user content verbatim.

## Submission and distribution

Publishing this independent repository does not authorize an upstream RFC/PR, organization transfer, suite trademark use, changes to another application or use of official signing secrets. Follow the user's explicit task and the team's decisions. Preview packages must preserve previous versions and include licenses/checksums. Read the adopted contribution process again before an actual application submission.
