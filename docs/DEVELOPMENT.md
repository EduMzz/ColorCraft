# Development and reproducible builds

## Layout

| Path | Responsibility |
|---|---|
| `crates/core` | Project model, color math, relationships/solver, commands, diagnostics, capture sampling and exports. |
| `crates/storage` | Settings, project persistence, migration, recovery and public library publication. |
| `crates/desktop` | Native egui/eframe application, panels, dialogs, capture window and receiver integration. |
| `crates/bridge` | Versioned interchange types, validation, native file queue and receiver exclusion. |
| `examples` / `schemas` | Small redistributable examples and file-format schemas. |
| `tools` | Windows packaging, pinned dependency notices, PDFium and per-user registration scripts. |

Dependencies are locked by `Cargo.lock`. Craft crates are pinned to a Git revision; changing that pin requires compatibility checks. The bridge has no GUI dependency. Original commands own project mutations and undo/redo.

## Windows x64 prerequisites

- Git and Rust 1.95+ with the `x86_64-pc-windows-msvc` toolchain.
- Visual Studio Build Tools: Desktop development with C++, MSVC and the Windows SDK.
- A graphics driver supporting the application's OpenGL renderer.
- PowerShell and Windows `tar` for packaging; network access for Cargo dependencies and the pinned PDFium/license sources.

```powershell
cargo build --locked --release -p colorcraft
powershell -File tools/fetch-pdfium.ps1 -Destination target/release
./target/release/colorcraft.exe
```

`.cargo/config.toml` selects the static C/C++ runtime for Windows x64. PDFium is dynamically loaded and must match the pinned API version. `fetch-pdfium.ps1` verifies its archive SHA-256 and copies the DLL plus notices; it is not an unverified runtime download at application startup.

## Checks

Run from the repository root:

```powershell
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
powershell -File tools/fetch-pdfium.ps1 -Destination target/debug/deps
cargo test --locked --workspace
rustup target add wasm32-unknown-unknown
cargo check --locked -p colorcraft-core --target wasm32-unknown-unknown
```

PDF renderer tests need the DLL beside their test executable, hence `target/debug/deps`. For `cargo run` during desktop development, also prepare `target/debug`. The WebAssembly command checks the core only; it does not build a browser application or enable the local bridge in a browser.

The repository's CI runs these Windows/core checks. It does not stand in for the suite's shared release workflows, physical-device color calibration, all native platforms or a complete accessibility review.

## Package and install your build

```powershell
powershell -File tools/package.ps1
# Existing version folder already present:
powershell -File tools/package.ps1 -OutputTag preview-2
```

The script builds the executable, generates examples, collects notices and the pinned PDFium runtime, and writes a new portable folder/ZIP under `dist`. It refuses an existing output folder. `SHA256SUMS.txt` covers package files; the `.zip.sha256` file covers the ZIP. `-SkipBuild` is for an already-built, validated release executable.

Extract the full package. Optional `Register-ColorCraftFiles.ps1` registers file icons/associations per user; `Unregister-ColorCraftFiles.ps1` removes the corresponding registration. `Register-ColorCraftProtocol.ps1` separately registers URI navigation and the compatible receiver path. Moving the package requires registering its new location. Keep `ColorCraftData` during updates.

Do not publish official suite installers or configure suite signing identities from this independent repository. Installer formats, app identity, signing, catalog admission and adoption of `craft-common-actions` require agreement with the team.

## Work safely with examples

Use synthetic documents and isolated data when testing. A screenshot invocation selects isolated settings/data and disables normal autosave:

```powershell
./target/release/colorcraft.exe examples/Studio.colorcraft --screenshot preview.png --qa-language en-US --qa-theme dark --qa-size 1440x920
```

Explicit test-only roots are documented in [BRIDGE-v1.md](BRIDGE-v1.md). Never point receiver tests or destructive scenarios at a user's real projects/library. Preserve existing installations and running instances. Record observed behavior and limits rather than converting an unrun check into a passing claim.

## Extending the application

See [ADDING-RELATIONS.md](ADDING-RELATIONS.md) for the relation/recipe registry and [the roadmap](../ROADMAP.md) for the future formula editor. Treat numerical changes as solver changes: protect fixed values, preserve atomic rollback and retain typed provenance for diagnostics. UI rendering alone must not change a project's colors.

The [application submission brief](ARTCRAFT-PROPOSAL.md) records compatibility and governance gaps. Keep current capabilities, experiments and planned features distinct in documentation.
