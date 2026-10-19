# ColorCraft 0.1.11

- Home uses the same Lucide house asset as VectorCraft. Workspace switcher uses a centered 22-point surface inside the 32-point application bar, with a vector chevron and compact-window behavior.
- Removed detach buttons from panel headers. Dragging and docking remain available.
- Tree insertion follows selection for add/paste: selected folder, selected color's parent, or parent of the shallowest selected item. Card navigation keeps its current folder as destination.
- Redesigned Relate Colors, Preferences, and relationship cards. Preferences has Interface, Files and Saving, and Workspace sections; dialog actions remain visible in reduced windows.
- Added Semantic Roles panel with a permanent default binding and optional mode bindings. Bind by dragging one color from the grid/tree or using the color selector. Empty bindings persist and undo/redo atomically.
- Project modes are managed in Details when the palette root is selected; a role can also create a mode while adding its binding.
- Tags use removable chips and a searchable popup. Enter reuses an existing tag or creates the typed tag. Tag vocabulary comes from the current palette.
- Deleting an associated color keeps the semantic role and clears affected bindings. CSS explicitly unsets empty mode bindings with `initial`; native project/palette formats preserve them.
- New interface labels and accessible icon names support Portuguese and English. User-authored names and notes remain unchanged.
