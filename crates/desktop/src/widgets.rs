//! Small original vector marks. No font-dependent pictograms or third-party artwork.
use eframe::egui::{self, Pos2, Stroke, Vec2};
pub struct TitleEdit {
    pub project: String,
    pub item: Option<String>,
    pub value: String,
    pub focus: bool,
}
impl crate::app::ColorCraftApp {
    pub fn editable_title(
        &mut self,
        ui: &mut egui::Ui,
        label: &str,
        item: Option<String>,
        size: f32,
    ) {
        let project = self.project().map(|p| p.id.clone()).unwrap_or_default();
        let editing = self
            .title_edit
            .as_ref()
            .is_some_and(|e| e.project == project && e.item == item);
        let mut commit = false;
        let mut cancel = false;
        let mut begin = false;
        ui.horizontal(|ui| {
            if editing {
                if let Some(edit) = &mut self.title_edit {
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut edit.value)
                            .font(egui::FontId::proportional(size))
                            .desired_width((ui.available_width() - 72.0).clamp(60.0, 450.0)),
                    );
                    if edit.focus {
                        response.request_focus();
                        edit.focus = false;
                    }
                    cancel = ui.input(|i| i.key_pressed(egui::Key::Escape));
                    commit = ui.input(|i| i.key_pressed(egui::Key::Enter)) || response.lost_focus();
                    commit |= crate::icons::button(ui, "check", "Confirmar nome").clicked();
                    cancel |=
                        crate::icons::button(ui, "close", "Cancelar edição do nome").clicked();
                }
            } else {
                let width = (ui.available_width() - 38.0).max(30.0);
                let label_width = ui
                    .painter()
                    .layout_no_wrap(
                        label.into(),
                        egui::FontId::proportional(size),
                        ui.visuals().text_color(),
                    )
                    .size()
                    .x
                    .min(width);
                ui.add_sized(
                    [label_width, size + 8.0],
                    egui::Label::new(egui::RichText::new(label).size(size))
                        .truncate()
                        .wrap_mode(egui::TextWrapMode::Truncate),
                );
                begin = crate::icons::enabled_button(
                    ui,
                    self.picking.is_none(),
                    "edit",
                    if item.is_some() {
                        "Editar nome da cor"
                    } else {
                        "Editar título da paleta"
                    },
                )
                .clicked();
            }
        });
        if cancel {
            self.title_edit = None;
        } else if commit && let Some(edit) = self.title_edit.take() {
            let value = edit.value.trim().to_string();
            if !value.is_empty()
                && value != label
                && let Some(p) = self.project()
            {
                let command = if let Some(id) = edit.item {
                    p.item(&id).cloned().map(|mut i| {
                        i.name = value;
                        colorcraft_core::engine::Command::Metadata(i)
                    })
                } else {
                    Some(colorcraft_core::engine::Command::ProjectDetails {
                        name: value,
                        description: p.description.clone(),
                    })
                };
                if let Some(command) = command {
                    self.issue(crate::app::Action::Core(command));
                }
            }
        } else if begin {
            self.title_edit = Some(TitleEdit {
                project,
                item,
                value: label.into(),
                focus: true,
            });
        }
    }
}
pub fn folder(ui: &egui::Ui, rect: egui::Rect) {
    let r = rect.shrink2(Vec2::new(rect.width() * 0.32, rect.height() * 0.32));
    let p = vec![
        r.left_bottom(),
        r.left_top(),
        r.left_top() + Vec2::new(r.width() * 0.38, 0.0),
        r.left_top() + Vec2::new(r.width() * 0.52, r.height() * 0.2),
        r.right_top() + Vec2::new(0.0, r.height() * 0.2),
        r.right_bottom(),
    ];
    ui.painter().add(egui::Shape::closed_line(
        p,
        Stroke::new(2.0, ui.visuals().weak_text_color()),
    ));
}
pub fn check(ui: &egui::Ui, center: Pos2) {
    ui.painter().add(egui::Shape::line(
        vec![
            center + Vec2::new(-4.0, 0.0),
            center + Vec2::new(-1.0, 3.0),
            center + Vec2::new(4.5, -3.0),
        ],
        Stroke::new(1.5, egui::Color32::WHITE),
    ));
}
pub fn arrow(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(22.0, 28.0), egui::Sense::hover());
    let a = rect.center() - Vec2::new(7.0, 0.0);
    let b = rect.center() + Vec2::new(7.0, 0.0);
    let stroke = Stroke::new(1.4, ui.visuals().text_color());
    ui.painter().line_segment([a, b], stroke);
    ui.painter()
        .line_segment([b - Vec2::new(4.0, 4.0), b], stroke);
    ui.painter()
        .line_segment([b - Vec2::new(4.0, -4.0), b], stroke);
}

pub fn tag_field(
    ui: &mut egui::Ui,
    id: egui::Id,
    tags: &mut Vec<String>,
    available: &[String],
) -> bool {
    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
    let escape_pressed = ui.input(|i| i.key_pressed(egui::Key::Escape));
    let mut changed = false;
    let mut query = ui
        .ctx()
        .data(|d| d.get_temp::<String>(id))
        .unwrap_or_default();
    let mut remove = None;
    let mut input = None;
    let mut open = ui
        .ctx()
        .data(|d| d.get_temp::<bool>(id.with("open")))
        .unwrap_or(false);
    let field = egui::Frame::new()
        .fill(ui.visuals().extreme_bg_color)
        .stroke(ui.visuals().widgets.inactive.bg_stroke)
        .corner_radius(6)
        .inner_margin(6)
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (index, tag) in tags.iter().enumerate() {
                    egui::Frame::new()
                        .fill(ui.visuals().selection.bg_fill)
                        .corner_radius(12)
                        .inner_margin(egui::Margin::symmetric(8, 2))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(tag).small());
                                let (rect, _) =
                                    ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::hover());
                                if crate::icons::at(ui, rect, "close", "Remover tag", false)
                                    .clicked()
                                {
                                    remove = Some(index);
                                }
                            });
                        });
                }
                let r = ui.add(
                    egui::TextEdit::singleline(&mut query)
                        .id(id.with("input"))
                        .frame(egui::Frame::NONE)
                        .desired_width(ui.available_width().max(90.0))
                        .hint_text(crate::i18n::tr("Adicionar tag…")),
                );
                if r.has_focus() {
                    open = !query.trim().is_empty();
                }
                input = Some(r);
            });
        });
    if let Some(index) = remove {
        tags.remove(index);
        changed = true;
    }
    let Some(response) = input else {
        return changed;
    };
    let mut choose = None;
    if (response.has_focus() || response.lost_focus()) && enter_pressed && !query.trim().is_empty()
    {
        choose = Some(
            available
                .iter()
                .find(|t| t.eq_ignore_ascii_case(query.trim()))
                .cloned()
                .unwrap_or_else(|| query.trim().to_owned()),
        );
    }
    if response.has_focus() && escape_pressed {
        open = false;
    }
    let mut anchor = response.clone();
    anchor.rect = field.response.rect;
    egui::Popup::from_response(&anchor)
        .open_bool(&mut open)
        .width(field.response.rect.width())
        .show(|ui| {
            ui.set_min_width((field.response.rect.width() - 16.0).max(80.0));
            egui::ScrollArea::vertical()
                .max_height(180.0)
                .show(ui, |ui| {
                    let needle = query.trim().to_lowercase();
                    for option in available
                        .iter()
                        .filter(|t| t.to_lowercase().contains(&needle) && !tags.contains(t))
                    {
                        if ui.selectable_label(false, option).clicked() {
                            choose = Some(option.clone());
                        }
                    }
                    if !needle.is_empty()
                        && !available
                            .iter()
                            .any(|t| t.eq_ignore_ascii_case(query.trim()))
                        && ui
                            .button(crate::i18n::tr(format!("Criar tag: {}", query.trim())))
                            .clicked()
                    {
                        choose = Some(query.trim().to_owned());
                    }
                });
        });
    if let Some(tag) = choose {
        if !tags.iter().any(|t| t.eq_ignore_ascii_case(&tag)) {
            tags.push(tag);
            changed = true;
        }
        query.clear();
        open = false;
        response.request_focus();
    }
    ui.ctx().data_mut(|d| {
        d.insert_temp(id, query);
        d.insert_temp(id.with("open"), open);
    });
    changed
}

#[cfg(test)]
mod tag_tests {
    use super::*;
    #[test]
    fn enter_creates_tag_chips_and_reuses_existing_spelling_without_duplicates() {
        let ctx = egui::Context::default();
        let id = egui::Id::new("tag-input-test");
        let mut tags = vec![];
        let options = vec!["Brand".to_string()];
        for text in ["brand", "Accent", "Brand"] {
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                tag_field(ui, id, &mut tags, &options);
            });
            output.textures_delta.clear();
            ctx.memory_mut(|m| m.request_focus(id.with("input")));
            let mut output = ctx.run_ui(
                egui::RawInput {
                    events: vec![
                        egui::Event::Text(text.into()),
                        egui::Event::Key {
                            key: egui::Key::Enter,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: Default::default(),
                        },
                    ],
                    focused: true,
                    ..Default::default()
                },
                |ui| {
                    tag_field(ui, id, &mut tags, &options);
                },
            );
            output.textures_delta.clear();
        }
        assert_eq!(tags, vec!["Brand", "Accent"]);
    }
}
