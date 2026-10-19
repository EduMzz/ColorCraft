//! ColorCraft presentation adapter. Layout transactions and serialization remain craft-ui's.
pub use craft_ui::docking::{
    Action, DockError, DropProposal, DropTarget, Layout, MAX_PANELS, Node, Permissions, Placement,
    Zone,
};
pub use surface::{DockArea, DockStyle, PanelLimits, cancel_drag};
use targets::resolve_drop;

// Retain the upstream renderer's adapter hooks for future custom headers.
#[allow(dead_code)]
mod surface;
mod targets;
