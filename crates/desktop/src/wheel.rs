use colorcraft_core::color;
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, TextureHandle, Vec2};
pub fn rgb(rgb: [f64; 3]) -> Color32 {
    Color32::from_rgb(
        rgb[0].clamp(0.0, 255.0).round() as u8,
        rgb[1].clamp(0.0, 255.0).round() as u8,
        rgb[2].clamp(0.0, 255.0).round() as u8,
    )
}
pub fn value_bar(ui: &mut egui::Ui, value: [f64; 3], enabled: bool) -> Option<([f64; 3], bool)> {
    let [h, s, v] = color::hsv(value);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(crate::i18n::tr("Claridade"))
                .small()
                .weak(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{:.1}%", v * 100.0))
                    .small()
                    .monospace(),
            );
        });
    });
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 28.0),
        if enabled {
            Sense::click_and_drag()
        } else {
            Sense::hover()
        },
    );
    response.widget_info(|| egui::WidgetInfo::slider(enabled, v * 100.0, "Claridade · V (HSV)"));
    #[cfg(test)]
    ui.ctx()
        .data_mut(|d| d.insert_temp(egui::Id::new("test-value-bar"), rect));
    let track = Rect::from_center_size(
        rect.center(),
        Vec2::new((rect.width() - 18.0).max(1.0), 8.0),
    );
    let mut mesh = egui::Mesh::default();
    let end = rgb(color::from_hsv([h, s, 1.0]));
    mesh.colored_vertex(track.left_top(), Color32::BLACK);
    mesh.colored_vertex(track.right_top(), end);
    mesh.colored_vertex(track.right_bottom(), end);
    mesh.colored_vertex(track.left_bottom(), Color32::BLACK);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    ui.painter().add(egui::Shape::mesh(mesh));
    ui.painter().rect_stroke(
        track,
        4,
        Stroke::new(0.7, ui.visuals().widgets.noninteractive.bg_stroke.color),
        egui::StrokeKind::Outside,
    );
    let at = Pos2::new(track.left() + track.width() * v as f32, track.center().y);
    ui.painter().circle_filled(
        at,
        7.0,
        if enabled {
            rgb(value)
        } else {
            ui.visuals().weak_text_color()
        },
    );
    ui.painter().circle_stroke(
        at,
        7.0,
        Stroke::new(
            2.0,
            if response.has_focus() {
                crate::theme::ACCENT
            } else {
                Color32::WHITE
            },
        ),
    );
    let mut next = None;
    if enabled
        && (response.clicked() || response.dragged())
        && let Some(pos) = response.interact_pointer_pos()
    {
        next = Some(((pos.x - track.left()) / track.width()).clamp(0.0, 1.0) as f64);
    }
    if enabled && response.has_focus() {
        ui.input(|input| {
            if input.key_pressed(egui::Key::ArrowRight) || input.key_pressed(egui::Key::ArrowUp) {
                next = Some((v + 0.01).min(1.0));
            }
            if input.key_pressed(egui::Key::ArrowLeft) || input.key_pressed(egui::Key::ArrowDown) {
                next = Some((v - 0.01).max(0.0));
            }
            if input.key_pressed(egui::Key::Home) {
                next = Some(0.0);
            }
            if input.key_pressed(egui::Key::End) {
                next = Some(1.0);
            }
        });
    }
    response.clone().on_hover_text(
        "Valor V do HSV: preserva matiz e saturação. Os vínculos e limites continuam ativos.",
    );
    next.filter(|next| (next - v).abs() > 1e-9)
        .map(|v| (color::from_hsv([h, s, v]), response.dragged()))
}
pub fn texture(ctx: &egui::Context) -> TextureHandle {
    let n = 384;
    let mut image = egui::ColorImage::filled([n, n], Color32::TRANSPARENT);
    for y in 0..n {
        for x in 0..n {
            let dx = (x as f64 + 0.5 - n as f64 / 2.0) / (n as f64 / 2.0 - 2.0);
            let dy = (y as f64 + 0.5 - n as f64 / 2.0) / (n as f64 / 2.0 - 2.0);
            let radius = dx.hypot(dy);
            if radius <= 1.0 {
                image[(x, y)] = rgb(color::from_hsv([
                    color::wrap((-dy).atan2(dx).to_degrees()),
                    radius,
                    1.0,
                ]));
            }
        }
    }
    ctx.load_texture("colorcraft-hsv-wheel", image, egui::TextureOptions::LINEAR)
}
pub fn show(
    ui: &mut egui::Ui,
    tex: &TextureHandle,
    value: [f64; 3],
    related: &[[f64; 3]],
    enabled: bool,
) -> Option<[f64; 3]> {
    let size = ui.available_width().clamp(250.0, 360.0);
    let (rect, response) = ui.allocate_exact_size(
        Vec2::splat(size),
        if enabled {
            Sense::click_and_drag()
        } else {
            Sense::hover()
        },
    );
    let center = rect.center();
    let radius = size / 2.0 - 3.0;
    ui.painter().image(
        tex.id(),
        rect,
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        if enabled {
            Color32::WHITE
        } else {
            Color32::from_white_alpha(90)
        },
    );
    let pos = |rgbv| {
        let [h, s, _] = color::hsv(rgbv);
        let a = h.to_radians() as f32;
        center + Vec2::new(a.cos(), -a.sin()) * (s as f32 * radius)
    };
    for other in related {
        let p = pos(*other);
        ui.painter().line_segment(
            [pos(value), p],
            Stroke::new(1.0, Color32::from_white_alpha(110)),
        );
        ui.painter().circle_filled(p, 4.0, rgb(*other));
        ui.painter()
            .circle_stroke(p, 5.0, Stroke::new(1.4, Color32::WHITE));
    }
    let p = pos(value);
    ui.painter()
        .circle_stroke(p, 9.5, Stroke::new(4.0, Color32::from_black_alpha(150)));
    ui.painter()
        .circle_stroke(p, 9.0, Stroke::new(2.0, Color32::WHITE));
    if enabled
        && (response.dragged() || response.clicked())
        && let Some(pos) = response.interact_pointer_pos()
    {
        let delta = pos - center;
        let saturation = (delta.length() / radius).clamp(0.0, 1.0) as f64;
        let hue = color::wrap((-delta.y as f64).atan2(delta.x as f64).to_degrees());
        let mut hsv = color::hsv(value);
        hsv[0] = hue;
        hsv[1] = saturation;
        return Some(color::from_hsv(hsv));
    }
    None
}
