# ColorCraft prototype — delivery validation

Validation observations: 2026-10-18; PoC scope clarified 2026-10-19. Windows x64; VectorCraft base 0.13.1 (`904f0fe5227bd987d55319e89a38b4b8f046998c`); companion ColorCraft 0.1.14. All mutation tests use synthetic documents and an isolated library/queue. Existing application installations, distributions and user palettes are not test inputs.

This fork is demonstration evidence for a possible suite integration, not the first upstream merge proposal. That proposal contains ColorCraft only. See [scope](ARTCRAFT-PROPOSAL.md). Results below do not establish production readiness or compatibility with every suite application.

## Automated checks

- Companion: formatting, Clippy with warnings denied and all **107** workspace tests passed. These include atomic receive/undo, native components and ICC bytes, dirty-state and moved-destination preflight, busy destinations, exact URI navigation, publication failure and replay across reload.
- Shared contract tests cover strict URI parsing, bounded JSON/profile data, source-profile conversion for sRGB, Display P3, Lab D50, CMYK and gray, unsupported profiles, exact profile-content comparison, receiver exclusion, malformed requests and durable request archives.
- Engine regressions cover explicit revision acceptance and undo, local object edits, missing/reappearing colors, archived ICC data, independent documents, malformed snapshots and automation file authority.
- UI regressions exercise tree/grid at narrow widths and 100%/150%, search ancestry and opening the sending form without document mutations.
- Final upstream CI result is recorded in the release verification section below.

## Native integration observations

- Opened the published fixture through the project picker. Applied linked fill/stroke to rectangles, including real grid pointer clicks and Alt targeting. Without a selection, the new default was used by subsequently created artwork.
- Sent selection colors, active color and a dragged proxy through the shared destination form. Existing native matches were indicated. Sending did not change the VectorCraft document or accepted revision.
- Changed a color in the ColorCraft interface. The destination form showed that other pending edits would also be saved. Send saved/published them; the VectorCraft artwork retained its previous values until explicit revision acceptance. One undo/redo restored/reapplied that accepted revision.
- Copy, unlink, project removal and undo were exercised. Temporarily removed the fixture library index: the accepted snapshot remained usable and the saved document reopened with its bindings.
- An open ColorCraft Preferences dialog produced the expected busy response. Missing-color navigation produced a contextual error. Hidden receiver queries did not require foreground activation.
- Automatic receiver start was exercised with ColorCraft closed. The isolated library override is inherited by the newly launched receiver. URI invocations were exercised with the receiver already running and closed; both opened the exact Coral entry and selected its `#9B50D8` value.
- Regression for **Edit in ColorCraft**: clicked the actual color context-menu item with ColorCraft closed. It started the registered native executable and selected Coral. With that receiver already open, navigation selected Ocean (`#1689B3`). The action no longer calls the browser launcher; an incompatible application-registration schema is rejected explicitly.
- A request queued while no receiver was running was processed after restart. Replaying that same saved batch after a second restart returned `alreadyApplied`, retained the same created IDs and kept the item count at 12; it did not append another color.
- Native selection collection included gradient stops, text colors and mesh points. Reducing object opacity to 25% left the collected native colors unchanged.
- Inspected the panel, picker, update comparison and sending form in English/Brazilian Portuguese, light/dark themes, 100%/150% interface scale, broad and reduced windows (including 900 × 750 physical pixels at 150%). The panel scrolls vertically; modal actions remain reachable. Existing upstream menu/status/property layouts can become crowded at the smallest whole-application dimensions.

## Original application compatibility

The original VectorCraft 0.13.1 CLI reopened and saved the synthetic native file without import/export warnings. The `colorcraft` extension, layers and global swatches were preserved. The prototype and original CLI PNG exports were pixel-identical across all **600 × 400** pixels. Both CLI exports used the same transparent-background policy; native preview screenshots include the white artboard and are not that comparison.

## Scope and limits

This is a local Windows integration prototype. Color-managed conversion checks establish numerical conversion and source-data retention, not physical display/print calibration. The original-application comparison is a synthetic document regression, not a guarantee for every third-party file. Full client browser compilation remains a separate pending check; the local file bridge is unavailable in the browser. macOS/Linux native integration, semantic-role/mode resolution, cross-computer synchronization and cloud storage are outside this delivery.

Detailed queue, document-extension, recovery, limits and commands: [COLORCRAFT-INTEGRATION.md](BRIDGE-v1.md).

## Release verification

Full client CI is not complete. The previous attempt passed formatting and Clippy, then stopped at a Windows linker PDB FILE_SYSTEM error. A retry advanced beyond that link but was stopped to bound local build resources. Do not treat it as a passing workspace test run or a production release gate.

The ten client engine integration tests passed when executed directly. A headless UI test exposed an unapplied-texture-delta cleanup issue; the harness was corrected and its targeted rerun is pending. Asset attribution (319 files), brand text, PDF dependency consistency, cursor inventory and layering (29 crates) passed separate checks. Full client WASM validation remains pending.

The companion 0.1.14 evaluation package was verified against 578 file hashes, its ZIP entries, ZIP sidecar and x64/static-CRT executable imports. Restart/replay and the direct native Edit action were verified as described above. These observations are PoC evidence, not an official VectorCraft release certification.
