//! Vector paths from the Colorator icon set (icons.mjs).
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
type Lines = Vec<(Vec<Pos2>, bool)>;
pub fn paint(ui: &egui::Ui, rect: Rect, name: &str, ink: Color32) {
    let key = egui::Id::new(("colorator-vector-icon", name));
    let lines = ui.ctx().data_mut(|data| {
        data.get_temp::<Lines>(key).unwrap_or_else(|| {
            let mut lines = vec![];
            let mut points = vec![];
            if let Ok(svg) = kurbo::BezPath::from_svg(path(name)) {
                kurbo::flatten(svg.iter(), 0.05, |el| match el {
                    kurbo::PathEl::MoveTo(p) => {
                        if !points.is_empty() {
                            lines.push((std::mem::take(&mut points), false));
                        }
                        points.push(Pos2::new(p.x as f32, p.y as f32));
                    }
                    kurbo::PathEl::LineTo(p) => points.push(Pos2::new(p.x as f32, p.y as f32)),
                    kurbo::PathEl::ClosePath if !points.is_empty() => {
                        lines.push((std::mem::take(&mut points), true));
                    }
                    _ => {}
                });
            }
            if !points.is_empty() {
                lines.push((points, false));
            }
            data.insert_temp(key, lines.clone());
            lines
        })
    });
    let side = rect.width().min(rect.height());
    let origin = rect.center() - Vec2::splat(side / 2.0);
    for (line, closed) in lines {
        let points = line
            .into_iter()
            .map(|p| origin + p.to_vec2() * (side / 24.0))
            .collect();
        let stroke = Stroke::new((side / 24.0 * 1.6).max(1.0), ink);
        ui.painter().add(if closed {
            egui::Shape::closed_line(points, stroke)
        } else {
            egui::Shape::line(points, stroke)
        });
    }
}
pub fn at(
    ui: &mut egui::Ui,
    rect: Rect,
    name: &str,
    label: &str,
    selected: bool,
) -> egui::Response {
    at_ink(ui, rect, name, label, selected, ui.visuals().text_color())
}
pub fn at_ink(
    ui: &mut egui::Ui,
    rect: Rect,
    name: &str,
    label: &str,
    selected: bool,
    ink: Color32,
) -> egui::Response {
    let label = crate::i18n::tr(label);
    let response = ui.place(
        rect,
        egui::Button::new("").selected(selected).frame(selected),
    );
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), &label)
    });
    if response.hovered() || response.has_focus() {
        ui.painter()
            .rect_filled(rect, 4, ui.visuals().widgets.hovered.bg_fill);
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect,
            4,
            Stroke::new(1.0, crate::theme::ACCENT),
            egui::StrokeKind::Inside,
        );
    }
    paint(
        ui,
        Rect::from_center_size(rect.center(), Vec2::splat(17.0)),
        name,
        ink,
    );
    response.on_hover_text(label)
}
pub fn button(ui: &mut egui::Ui, name: &str, label: &str) -> egui::Response {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(28.0), egui::Sense::hover());
    at(ui, rect, name, label, false)
}
pub fn enabled_button(ui: &mut egui::Ui, enabled: bool, name: &str, label: &str) -> egui::Response {
    ui.add_enabled_ui(enabled, |ui| button(ui, name, label))
        .inner
}
pub fn toggle(ui: &mut egui::Ui, name: &str, label: &str, selected: bool) -> egui::Response {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(28.0), egui::Sense::hover());
    at(ui, rect, name, label, selected)
}
pub fn caption(ui: &mut egui::Ui, name: &str, label: &str) -> egui::Response {
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(caption_width(ui.ctx()), 30.0),
        egui::Sense::hover(),
    );
    #[cfg(test)]
    ui.ctx()
        .data_mut(|d| d.insert_temp(egui::Id::new(("test-caption", name)), rect));
    let response = at(ui, rect, name, label, false);
    if name == "close" && response.hovered() {
        ui.painter()
            .rect_filled(rect, 0, Color32::from_rgb(196, 43, 49));
        paint(
            ui,
            Rect::from_center_size(rect.center(), Vec2::splat(17.0)),
            name,
            Color32::WHITE,
        );
    }
    response
}
pub fn caption_width(ctx: &egui::Context) -> f32 {
    if ctx.content_rect().width() < 720.0 {
        34.0
    } else {
        46.0
    }
}
pub fn text_button(ui: &mut egui::Ui, icon: &str, label: &str) -> egui::Response {
    text_toggle(ui, icon, label, false)
}
pub fn text_toggle(ui: &mut egui::Ui, icon: &str, label: &str, selected: bool) -> egui::Response {
    let label = crate::i18n::tr(label);
    let galley = ui.painter().layout_no_wrap(
        label.clone(),
        egui::TextStyle::Button.resolve(ui.style()),
        ui.visuals().text_color(),
    );
    let width = (galley.size().x + 44.0).min(ui.max_rect().width());
    let response = ui.add_sized([width, 28.0], egui::Button::new("").selected(selected));
    paint(
        ui,
        Rect::from_center_size(
            response.rect.left_center() + Vec2::new(17.0, 0.0),
            Vec2::splat(16.0),
        ),
        icon,
        ui.visuals().text_color(),
    );
    ui.painter()
        .with_clip_rect(response.rect.shrink2(Vec2::new(5.0, 0.0)))
        .galley(
            response.rect.left_center() + Vec2::new(32.0, -galley.size().y / 2.0),
            galley,
            ui.visuals().text_color(),
        );
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), &label)
    });
    response.on_hover_text(label)
}
pub(crate) fn path(name: &str) -> &'static str {
    match name {
        "conflict" => "m13 2-8 12h6l-1 8 9-13h-6z",
        "grid" => "M3 3h7v7H3z M14 3h7v7h-7z M3 14h7v7H3z M14 14h7v7h-7z",
        "tree" => "M5 3v14h5 M5 8h5 M13 5h7v6h-7z M13 14h7v6h-7z",
        "folder" => "M3 7V5a1 1 0 0 1 1-1h5l2 3h9a1 1 0 0 1 1 1v11H3z",
        "plus" => "M12 5v14 M5 12h14",
        "minus" => "M5 12h14",
        "zoom" => "M15 15l6 6 M17 10a7 7 0 1 1-14 0 7 7 0 0 1 14 0 M10 7v6 M7 10h6",
        "dropper" => "M2 22l2-2 M4 20v-4l9-9 4 4-9 9z M11 5l8 8 M15 6l3-3a2 2 0 0 1 3 3l-3 3",
        "eraser" => "M3 15l10-11a2 2 0 0 1 3 0l5 5a2 2 0 0 1 0 3L12 22H8z M8 10l9 9 M12 22h10",
        "brush" => {
            "M9 14l8-11a2 2 0 0 1 4 3l-10 10z M9 14c-4-1-5 2-5 4 0 2-2 3-2 3s7 1 8-3c1-2 1-3-1-4"
        }
        "fit" => {
            "M4 9V4h5 M15 4h5v5 M20 15v5h-5 M9 20H4v-5 M4 4l6 6 M20 4l-6 6 M20 20l-6-6 M4 20l6-6"
        }
        "color" => {
            "M12 3a9 9 0 1 0 0 18h1a3 3 0 0 0 0-6h-1a2 2 0 0 1 0-4h3a4 4 0 0 0 0-8z M7 8h.01 M6 12h.01 M10 6h.01"
        }
        "chevron" => "m9 5 7 7-7 7",
        "down" => "m5 9 7 7 7-7",
        "more" => "M5 12h.01 M12 12h.01 M19 12h.01",
        "close" => "m6 6 12 12 M6 18 18 6",
        "check" => "m5 12 4 4L19 6",
        "copy" => "M9 9h12v12H9z M15 9V3H3v12h6",
        "trash" => "M3 6h18 M9 6V3h6v3 M6 6l1 15h10l1-15 M10 10v7 M14 10v7",
        "edit" => "m4 16-1 5 5-1L21 7l-4-4z M14 6l4 4",
        "link" => {
            "m10 13 4-4 M8 15l-2 2a3.5 3.5 0 0 1-5-5l5-5a3.5 3.5 0 0 1 5 0 M13 9l2-2a3.5 3.5 0 0 1 5 5l-5 5a3.5 3.5 0 0 1-5 0"
        }
        "unlink" => "m3 3 18 18 M9 5l2-2a4 4 0 0 1 6 6 M15 19l-2 2a4 4 0 0 1-6-6",
        "sun" => {
            "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8 M12 2v2 M12 20v2 M2 12h2 M20 12h2 M5 5l2 2 M17 17l2 2 M5 19l2-2 M17 7l2-2"
        }
        "moon" => "M20.9 13a9 9 0 1 1-9.9-9.9 7 7 0 0 0 9.9 9.9",
        "undo" => "M9 4 3 10l6 6 M3 10h11a6 6 0 0 1 0 12",
        "redo" => "m15 4 6 6-6 6 M21 10H10a6 6 0 0 0 0 12",
        "download" => "M12 3v12 m-5-5 5 5 5-5 M4 16v5h16v-5",
        "upload" => "M12 16V4 m-5 5 5-5 5 5 M4 16v5h16v-5",
        "search" => "M10 3a7 7 0 1 0 0 14 7 7 0 0 0 0-14 m5 12 6 6",
        "code" => "m8 5-6 7 6 7 M16 5l6 7-6 7 M14 3l-4 18",
        // Lucide house, as used by VectorCraft; ISC notice in docs/LUCIDE-ICONS-LICENSE.txt.
        "home" => {
            "M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8 M3 10a2 2 0 0 1 .709-1.528l7-6a2 2 0 0 1 2.582 0l7 6A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"
        }
        "settings" => "M3 6h18 M3 12h18 M3 18h18 M7 3v6 M17 9v6 M9 15v6",
        "roles" => "M4 4h6v6H4z M14 14h6v6h-6z M7 10v7h7 M14 4h6v6h-6z M10 7h4",
        "tag" => "M3 3h8l10 10-8 8L3 11z M7 7h.01",
        "contrast" => "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18 M12 3v18 M15 5v14 M18 7v10",
        "modes" => "M4 4h12v12H4z M8 20h12V8",
        "file" => "M4 2h10l6 6v14H4z M14 2v6h6 M8 13h8 M8 17h8",
        "save" => "M3 3h15l3 3v15H3z M7 3v6h10V3 M7 21v-8h10v8",
        "arrow" => "M4 12h16 m-6-6 6 6-6 6",
        "cut" => {
            "m7 7 14 14 M7 17 21 3 M5 3a3 3 0 1 0 0 6 3 3 0 0 0 0-6 M5 15a3 3 0 1 0 0 6 3 3 0 0 0 0-6"
        }
        "paste" => "M9 4H4v18h16V4h-5 M9 2h6v5H9z",
        "refresh" => "M20 7a9 9 0 1 0 1 8 M20 2v6h-6",
        "eye" => "M2 12s4-7 10-7 10 7 10 7-4 7-10 7-10-7-10-7 M12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6",
        "move" => "M12 2v20 M2 12h20 m-13-7 3-3 3 3 m-6 14 3 3 3-3 M5 9l-3 3 3 3 M19 9l3 3-3 3",
        "lock" => "M5 11h14v10H5z M8 11V7a4 4 0 1 1 8 0v4",
        "unlock" => "M5 11h14v10H5z M8 11V7a4 4 0 0 1 7-3",
        "spark" => "m12 2 3 7 7 3-7 3-3 7-3-7-7-3 7-3z",
        "info" => "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18 M12 10v7 M12 7v.01",
        "minimize" => "M5 12h14",
        "maximize" => "M5 5h14v14H5z",
        "restore" => "M8 8V4h12v12h-4 M4 8h12v12H4z",
        "external" => "M14 3h7v7 M21 3 10 14 M10 3H3v18h18v-7",
        "gradient" => "M3 5h18v14H3z M7 5v14 M11 5v14 M15 5v14",
        "hand" => {
            "M8 13V5a2 2 0 0 1 4 0v7 M12 5a2 2 0 0 1 4 0v7 M16 7a2 2 0 0 1 4 0v9c0 4-3 6-7 6-3 0-5-2-6-4l-4-6a2 2 0 0 1 3-2l2 3 M8 7V4a2 2 0 0 1 4 0"
        }
        "hand-blocked" => {
            "M8 13V5a2 2 0 0 1 4 0v7 M12 5a2 2 0 0 1 4 0v7 M16 7a2 2 0 0 1 4 0v9c0 4-3 6-7 6-3 0-5-2-6-4l-4-6a2 2 0 0 1 3-2l2 3 M8 7V4a2 2 0 0 1 4 0 M3 3l18 18"
        }
        _ => "M6 6l12 12 M6 18L18 6",
    }
}
