use super::{DropProposal, DropTarget, Placement, Zone};
use eframe::egui::{Pos2, Rect, Vec2};

pub(super) fn usable(rect: Rect) -> bool {
    rect.is_finite()
        && rect.width() > 0.0
        && rect.height() > 0.0
        && rect.width() <= 1_000_000.0
        && rect.height() <= 1_000_000.0
}

fn front<P>(pointer: Pos2, targets: &[DropTarget<P>]) -> Option<&DropTarget<P>> {
    if !pointer.is_finite() || targets.len() > super::MAX_PANELS {
        return None;
    }
    targets
        .iter()
        .rev()
        .find(|t| usable(t.rect) && t.rect.contains(pointer))
}

/// Visible cross of small targets; the remainder of a panel's body stays free for floating.
pub(super) fn buttons<P: Eq>(
    panel: &P,
    pointer: Pos2,
    targets: &[DropTarget<P>],
) -> Vec<(Zone, Rect)> {
    let Some(target) = front(pointer, targets) else {
        return vec![];
    };
    if target.tabs.len() > super::MAX_PANELS
        || (&target.anchor == panel && !target.tabs.iter().any(|(p, _)| p != panel))
    {
        return vec![];
    }
    let top = target
        .tab_strip
        .filter(|r| usable(*r))
        .map_or(target.rect.top(), |r| r.bottom());
    let body = Rect::from_min_max(Pos2::new(target.rect.left(), top), target.rect.max);
    let size = ((body.width().min(body.height()) - 16.0) / 3.0).clamp(8.0, 30.0);
    let step = size + 4.0;
    let center = body.center();
    let mut buttons = vec![];
    if target.accepts_tabs && !target.tabs.iter().any(|(p, _)| p == panel) {
        buttons.push((
            Zone::Center,
            Rect::from_center_size(center, Vec2::splat(size)),
        ));
    }
    if target.allow_splits {
        for (zone, offset) in [
            (Zone::Left, Vec2::new(-step, 0.0)),
            (Zone::Right, Vec2::new(step, 0.0)),
            (Zone::Top, Vec2::new(0.0, -step)),
            (Zone::Bottom, Vec2::new(0.0, step)),
        ] {
            buttons.push((
                zone,
                Rect::from_center_size(center + offset, Vec2::splat(size)),
            ));
        }
    }
    buttons
}

pub(super) fn resolve_drop<P: Clone + Eq>(
    panel: &P,
    pointer: Pos2,
    targets: &[DropTarget<P>],
) -> Option<DropProposal<P>> {
    let target = front(pointer, targets)?;
    if target
        .tab_strip
        .is_some_and(|strip| usable(strip) && strip.contains(pointer))
    {
        // Preserve shared tab insertion, ordering and frontmost-group occlusion.
        return craft_ui::docking::resolve_drop(panel, pointer, targets);
    }
    let zone = buttons(panel, pointer, targets)
        .into_iter()
        .find(|(_, r)| r.contains(pointer))?
        .0;
    let anchor = if &target.anchor == panel {
        &target.tabs.iter().find(|(p, _)| p != panel)?.0
    } else {
        &target.anchor
    };
    Some(DropProposal {
        anchor: anchor.clone(),
        placement: Placement::Split(zone),
        preview: craft_ui::docking::zone_rect(target.rect, zone)?,
        insertion: None,
    })
}
