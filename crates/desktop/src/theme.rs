//! ColorCraft surfaces and controls, using the same widget mechanism as Craft.
use eframe::egui::{self, Color32, Stroke, Vec2};
pub const ACCENT: Color32 = Color32::from_rgb(242, 190, 69);
pub const APP_BAR: f32 = 32.0;
pub const TAB_BAR: f32 = 32.0;
pub fn accent_text(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        ACCENT
    } else {
        Color32::from_rgb(137, 90, 0)
    }
}
pub fn apply(ctx: &egui::Context, dark: bool, scale: f32) {
    let mode = if dark {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    };
    ctx.set_theme(mode);
    ctx.set_zoom_factor(scale.clamp(0.8, 1.5));
    ctx.style_mut_of(mode, |s| {
        s.spacing.item_spacing = Vec2::new(8.0, 7.0);
        s.spacing.button_padding = Vec2::new(10.0, 5.0);
        s.spacing.interact_size.y = 28.0;
        s.spacing.scroll = egui::style::ScrollStyle::solid();
        s.text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(13.0));
        s.text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(13.0));
        s.text_styles
            .insert(egui::TextStyle::Small, egui::FontId::proportional(11.0));
        s.text_styles
            .insert(egui::TextStyle::Heading, egui::FontId::proportional(17.0));
        let (surface, field, button, hover, border, text) = if dark {
            (
                Color32::from_rgb(30, 32, 37),
                Color32::from_rgb(23, 25, 29),
                Color32::from_rgb(43, 46, 52),
                Color32::from_rgb(53, 57, 64),
                Color32::from_rgb(63, 67, 75),
                Color32::from_rgb(230, 232, 237),
            )
        } else {
            (
                Color32::from_rgb(244, 245, 247),
                Color32::WHITE,
                Color32::from_rgb(233, 235, 239),
                Color32::from_rgb(219, 223, 229),
                Color32::from_rgb(192, 197, 205),
                Color32::from_rgb(36, 40, 48),
            )
        };
        craft_ui::theme::WidgetPalette {
            noninteractive: surface,
            inactive: button,
            hovered: hover,
            active: hover,
            open: hover,
        }
        .apply_to(
            &mut s.visuals.widgets,
            egui::CornerRadius::same(4),
            Stroke::new(1.0, text),
        );
        s.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
        s.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, border);
        s.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, border);
        s.visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
        s.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border);
        s.visuals.panel_fill = surface;
        s.visuals.window_fill = surface;
        s.visuals.extreme_bg_color = field;
        s.visuals.faint_bg_color = button;
        s.visuals.override_text_color = Some(text);
        s.visuals.selection.bg_fill = if dark {
            Color32::from_rgb(78, 66, 36)
        } else {
            Color32::from_rgb(255, 235, 183)
        };
        s.visuals.selection.stroke = Stroke::new(
            1.0,
            if dark {
                ACCENT
            } else {
                Color32::from_rgb(132, 88, 0)
            },
        );
        s.visuals.window_corner_radius = egui::CornerRadius::same(10);
        s.visuals.menu_corner_radius = egui::CornerRadius::same(8);
        s.visuals.window_stroke = Stroke::new(1.0, border);
    });
}
pub fn primary(label: &str) -> egui::Button<'_> {
    egui::Button::new(
        egui::RichText::new(crate::i18n::tr(label))
            .strong()
            .color(Color32::from_rgb(30, 27, 20)),
    )
    .fill(ACCENT)
    .stroke(Stroke::NONE)
}
pub fn card(ui: &egui::Ui) -> egui::Frame {
    egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(8)
        .inner_margin(12)
}
pub fn section(ui: &mut egui::Ui, icon: &str, title: &str) {
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(Vec2::splat(20.0), egui::Sense::hover());
        crate::icons::paint(ui, r.shrink(1.0), icon, ui.visuals().text_color());
        ui.label(egui::RichText::new(crate::i18n::tr(title)).strong());
    });
}
/// Consistent heading typography for top-level dock panels.
pub fn panel_title(ui: &mut egui::Ui, title: &str) -> egui::Response {
    ui.label(
        egui::RichText::new(crate::i18n::tr(title))
            .size(18.0)
            .strong(),
    )
}
pub fn chips<T: PartialEq + Clone>(
    ui: &mut egui::Ui,
    current: &mut T,
    choices: &[(T, &str)],
) -> bool {
    let mut changed = false;
    ui.horizontal_wrapped(|ui| {
        for (value, label) in choices {
            if ui
                .add(
                    egui::Button::new(*label)
                        .selected(current == value)
                        .corner_radius(14),
                )
                .clicked()
                && current != value
            {
                *current = value.clone();
                changed = true;
            }
        }
    });
    changed
}
