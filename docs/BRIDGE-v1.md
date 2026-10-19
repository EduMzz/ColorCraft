# ColorCraft integration prototype — v1

**PoC scope:** this repository demonstrates a possible client integration for suite applications. The initial merge proposal is ColorCraft only; the VectorCraft adapter and document extension remain here as experiments. This contract is not an adopted suite standard. See [scope and merge boundaries](MERGE-PREPARATION.md).

Independent build by EduMzz, based on VectorCraft 0.13.1, commit `904f0fe5227bd987d55319e89a38b4b8f046998c`. Requires ColorCraft 0.1.14 or a receiver implementing this contract. No cloud, network service, automatic recoloring, or semantic-role resolution is involved.

## Windows quick start

1. Extract each new package into its own directory. Keep existing installations and their data folders.
2. In the ColorCraft 0.1.14 directory, run `Register-ColorCraftProtocol.ps1` with PowerShell. It registers only the per-user `colorcraft:` URL protocol and the receiver executable. It does not change file associations. Moving the package requires registering the new location.
3. Save a palette in ColorCraft. Its public snapshot is published to `%APPDATA%\ArtCraft\Palettes\library.json`.
4. In the prototype, open Window → ColorCraft Swatches, then Add Project. Choose a published palette.
5. Choose the fill/stroke proxy and click a color; Alt targets the other proxy. With no object selected, this changes the default for new artwork.

Grid folders open with a click; breadcrumbs return to ancestors. Tree folders expand. Search keeps matching folder paths. Context menus edit the exact source in ColorCraft, copy an independent document swatch, unlink, or remove the project. Removing/unlinking preserves artwork and color values. Used internal global swatches are hidden from the normal Swatches panel until unlinked.

The panel polls the library every two seconds. An update badge does **not** change the artwork. Review the changed, added and removed colors and accept explicitly. The entire accepted revision is one undo step. Missing colors retain the last accepted value and are marked unavailable. Saved documents keep their snapshot and ordinary global swatches, so the artwork remains readable without ColorCraft or the library.

## Sending colors

Use Send Active Color, drag a solid fill/stroke chip to a project/folder, or use the panel menu's Send Selection Colors. The selection collector includes solid fills/strokes, text runs, gradient stops and mesh points. Patterns/images are not rasterized. Object opacity is excluded. Exact repeated native colors in a batch are grouped; matching existing colors are identified and may be unchecked or added as new colors. Existing palette colors are never overwritten.

All routes use the same destination form. The receiver is inspected before confirmation. A warning states when the operation will also save pending ColorCraft edits. A changed project/folder/state rejects the old confirmation: refresh the destination and confirm again. An active ColorCraft gesture or dialog reports busy. Confirmation is performed in VectorCraft; no second approval dialog is required in ColorCraft.

The send is one ColorCraft undo operation, followed by save and publication. It does not link or recolor the originating objects. Accept the published revision separately in VectorCraft. If publication fails after saving, Retry Publication republishes without appending again. After an uncertain timeout, Check Previous Send queries/retries the **same** request ID. Do not start a new send to resolve an uncertain previous result.

## URI navigation

`colorcraft://open?palette=<palette-id>&color=<color-id>`

`color` is optional. Query components are percent encoded. Unknown/duplicate query parameters, paths, fragments and other schemes/hosts are rejected. The receiver resolves the exact palette through the library, reuses an open document, expands/reveals the exact color and focuses the window. A missing/moved file, wrong project ID, or missing color is an explicit error; no approximate project match is opened.

The protocol registration is per user in `HKCU\Software\Classes\colorcraft`. A normal compatible startup refreshes its executable registration. `--bridge-start` starts the receiver hidden; an `open` request makes it visible. A process-wide file lock elects one compatible receiver per user; secondary URI invocations enqueue then exit. Older incompatible application instances are left running.

The panel's **Edit in ColorCraft** action passes that URI directly to the compatible executable registered in `app.json`. It does not use the web browser launcher: browser helpers can treat custom schemes as web pages. External `colorcraft:` links still use the per-user protocol association. A missing application registration asks the user to choose the compatible executable from the panel menu.

## Public library and accepted document extension

The adapter reads ColorCraft's resolved `colorcraft-palette`, schema 1 snapshots only. Index paths must be relative, confined below the library directory (including canonical/symlink resolution), and identity/revision must match the index. JSON reads are bounded at 256 MiB, palettes at 10,000 items, hierarchy at 128 levels, 100 projects per VectorCraft document, ICC at 16 MiB each/64 MiB aggregate. Invalid/future versions preserve the accepted document state and display an error.

Native document root extension:

```json
{"colorcraft":{"schemaVersion":1,"projects":[{"snapshot":{"format":"colorcraft-palette","schemaVersion":1,"paletteId":"p","revision":1,"name":"Brand","profiles":[],"items":[]},"bindings":{"color-id":"@colorcraft:<digest>"},"unavailable":[],"unavailableProfiles":[]}]}}
```

`bindings` maps stable external color IDs to ordinary global swatches. `unavailable` archives removed bound items; `unavailableProfiles` retains their original ICC bytes under content-derived IDs. Native components and embedded source ICC bytes remain in the snapshot. On first use/accepted revision, the source profile converts to Lab D50 for the global swatch, independently of document profile. The published sRGB preview is for UI. Unsupported conversion requires explicit approval of preview approximation or cancellation; approximation does not replace the source record. Local object edits follow normal VectorCraft unlink rules and later updates do not reclaim that object.

## Local file queue

Root: `%LOCALAPPDATA%\ColorCraft\Bridge\v1`.

- `app.json`: `{schemaVersion:1, executable:<absolute .exe>, version:<string>}`.
- `receiver.lock`: held exclusively for receiver lifetime.
- `requests/<requestId>.json`: immutable request, atomically renamed from a unique temporary file.
- `completed/<requestId>.json`: durable processed request archive; completed requests leave the active scan.
- `responses/<requestId>.json`: persistent atomic response. Retain request/response pairs for idempotence; do not reuse IDs with another payload.

No ports or listeners. Same-user local files are the trust boundary. Requests/responses are not encrypted and are not an authenticated boundary against other processes running as that user. Request reads are limited to 32 MiB and batches to 1,000 colors. Directory scans/process batches are bounded. Malformed individual requests receive errors without blocking later valid requests.

```json
{"schemaVersion":1,"requestId":"UUID","operation":"append","paletteId":"p","folderId":null,"expectedState":"opaque preflight token","language":"en","profiles":[],"colors":[{"name":"Ocean","color":{"space":"srgb","components":[0.1,0.4,0.8]},"profiles":[]}]}
```

Operations: `inspect` returns live snapshot, dirty flag and opaque state; `append` requires the confirmed state; `publish` retries a clean saved document's publication; `open` accepts optional `colorId`. Root `profiles` deduplicates ICC bytes; row profiles are accepted for a self-contained dragged color. Profile entries are `{id,name,bytes:[0..255]}`. Native spaces: `srgb`, `display-p3`, `lab` (D50, L 0–100, a/b −128–127), `cmyk`, `gray`; non-Lab components are 0–1. Profiles are preserved and remapped by content digest, never inferred from a display HEX.

Responses: `{schemaVersion,requestId,ok,status,error?,state?,dirty,snapshot?,colorIds}`. Statuses include `ready`, `opened`, `savedAndPublished`, `alreadyApplied`, `savedPublicationPending`, `published`, `busy`, `changed`, `rejected`, `invalidRequest`. A saved receipt is stored with the ColorCraft project as protection across restart between disk save and response write. Responses persist separately so replay never appends again. A save failure rolls the in-memory batch back. Undo/redo is owned by the editor that performed the mutation.

## Commands and extension points

Engine commands: `colorcraft.projects`, `colorcraft.project.add`, `colorcraft.apply`, `colorcraft.bind`, `colorcraft.project.update`, `colorcraft.color.copy`, `colorcraft.color.detach`, `colorcraft.project.remove`, `colorcraft.colors.collect`. `colorcraft.state.set` is the validated internal batch step. See each registered command's parameter documentation via `engine.commands`.

UI commands: `ui.colorcraft.addProject`, `.view`, `.refresh`, `.update`, `.send`, `.inspect`, `.retry`, `.publish`, `.edit`, `.configure`. All are reachable through the existing control channel/MCP. Automation must grant read authority for the library, source project and registered executable, and read/write authority for the queue; append/publication also require write authority for the project and library. The `colorcraftBridge` dialog has add/send/update modes. The bridge crate has no GUI dependency; engine commands own document mutations; the panel is an adapter. Both repositories carry the same contract crate source, separately formatted by their workspace.

Environment overrides for isolated tests: `COLORCRAFT_BRIDGE_DIR`, `COLORCRAFT_LIBRARY_DIR`; companion arguments `--bridge-dir`, `--library-dir`, `--data-dir`, `--config-dir`, `--isolated`. Never point QA fixtures at the real library. Browser UI disables local bridge actions while keeping portable global swatches and document extension data readable.

## Client PoC build, packaging and verification

The commands in this section belong to the VectorCraft PoC checkout, not the standalone ColorCraft workspace. ColorCraft build and packaging commands are in its README.

`cargo xtask ci` runs upstream formatting, Clippy, workspace tests, asset/brand/PDF dependency checks, layering and WASM. `cargo test -p vectorcraft-engine --test colorcraft_bridge` targets the adapter. `packaging/windows/colorcraft-prototype.ps1` produces a separately named portable folder and ZIP with original notices, dependency licenses and SHA-256 sums. Portable builds use optimized release code, static CRT and eight codegen units; LTO is disabled consistently for the prototype profile to bound link-time memory. No updater or installer migration targets official application profiles. See `BRIDGE-VALIDATION.md` for observed results and limitations of this delivery.
