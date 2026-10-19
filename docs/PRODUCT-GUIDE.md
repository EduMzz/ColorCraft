# ColorCraft: the basic workflow

## Start with a project

Create a palette from Home, or open `examples/Studio.colorcraft`. Each open document has its own undo history. Colors have stable IDs; folders organize them. Cards emphasize browsing, while the tree exposes hierarchy. Ctrl adds/removes individual selections; Shift selects a visible range.

Select a color to edit its wheel, numeric channels or HEX representation. The native color space is recorded separately from the sRGB display preview. Choose the interface language, theme and scale in Edit → Preferences.

## Turn individual colors into a system

Select two or more colors and open Relate Colors. A parametric link shares only the RGB/HSV channels you choose. Existing nonzero RGB/S/V values use ratios; zeros use offsets; hue uses a circular offset. Participants can be edited in either direction where the complete system permits it.

For example, Studio links Lagoon's hue to Iris and Marigold. Change Lagoon's hue and those related hues move with it. Their other channels remain independent. Iris is protected against direct editing, but can follow the relationship.

Harmonies generate a related family. Gradients create discrete colors between endpoints with a chosen interpolation. Creating new colors from a locked original is allowed when its value need not change. Replacing an existing color is a separate, explicit choice in the composition preview.

The current application provides built-in rules and recipes. A free-form formula/variable editor is planned, not hidden behind an advanced setting.

## Use constraints and protection deliberately

A limiting link expresses a boundary. The current type is minimum WCAG contrast between opaque sRGB colors. A drag can stop at the valid boundary; an incompatible numeric edit is refused. The other participant is not moved merely to compensate.

- The **hand** blocks direct numerical editing while permitting relationship-driven changes.
- The **padlock** keeps the value fixed, including changes that would arrive through relationships.
- A **source link** protects a captured color. Confirming an unlock detaches that link and preserves its current value.

The Conflicts panel explains refused/limited operations through chains of colors, relations and blockers. Its contextual unlink action removes a whole relationship while retaining the colors. It does not automatically repeat the failed edit. The conflict history is session-local and separate from the project's undo history.

## Capture from a source

Add an image, PDF or SVG in Sources and double-click it. The capture window has an image area, vertical tools, options and a capture list.

Click empty image space with the eyedropper to create a point capture. Click an existing marker to select it; drag it to reposition. Choose an exact pixel, neighborhood mean or median, and the area's shape/radius where applicable.

Use a new mask for an independent brush capture. Continue painting a selected mask to add strokes; the stroke eraser removes a whole intersected stroke. Hand and zoom move the view without changing capture coordinates.

Apply commits the draft and associated color changes as one undo step. Cancel discards the draft. Linked files remain external by default; embedding is explicit. Changed sources retain their current colors until an explicit update. A missing file needs relinking. Profile warnings require an explicit choice where fidelity cannot be guaranteed.

## Describe meaning without changing the values

Details stores names, descriptions, tags and custom metadata. Semantic Roles associates functions such as `text.primary` with colors. Each role has a default assignment and optional assignments for modes. Create modes in project-root Details or while adding a mode to a role.

Changing these assignments does not transform color values. A role can use Ink in the default mode and Paper in Dark mode without copying or recoloring either color. Documentary relations record usage, such as text on a surface, without becoming solver constraints.

## Save, publish and reuse

Save the `.colorcraft` project to retain editing rules, source information and semantic intent. Saving also publishes a resolved snapshot to `%APPDATA%\ArtCraft\Palettes\library.json`. This is a local per-user library, not an internet upload. A publication failure is reported separately from a successful project save.

Autosave can save previously named projects and update their public snapshots every 15 seconds. A new/imported unnamed project needs its first Save/Save As. Recovery drafts are a separate preference.

Export an interchange format for another program. CSS, DTCG, CSV and SVG are outputs rather than editable-project round trips. Check the format's conversion/loss notice. The experimental VectorCraft client retains its accepted palette revision until the user reviews and accepts a newer one.

See [the roadmap](../ROADMAP.md), [formats](FORMATOS.md), [bridge contract](BRIDGE-v1.md) and the more extensive [Portuguese manual](MANUAL.md).
