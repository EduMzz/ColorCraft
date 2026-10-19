//! Pair metrics and multi-colour parametric relationships.
use crate::{
    app::{Action, ColorCraftApp},
    dialogs::{Dialog, color_combo},
    icons, theme, wheel,
};
use colorcraft_core::{catalog, color, engine::Command, *};
use eframe::egui::{self, RichText, Vec2};
impl ColorCraftApp {
    pub fn link_content(&mut self, ui: &mut egui::Ui, dialog: &mut Dialog) -> bool {
        let Dialog::Link {
            participants,
            a,
            b,
            channels,
            kind,
            minimum,
            parametric_type,
            limit_type,
            convert,
            relation_kind,
            note,
        } = dialog
        else {
            return true;
        };
        let p = self.project().cloned().unwrap_or_default();
        let mut keep = true;
        let original_pair = (a.clone(), b.clone(), *kind);
        theme::section(ui, "link", "Relacionar cores");
        ui.label(
            RichText::new(crate::i18n::tr(
                "Escolha o tipo de relação e confira as cores participantes.",
            ))
            .small()
            .weak(),
        );
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            if icons::text_toggle(ui, "link", "Vínculo paramétrico", *kind == 0).clicked() {
                *kind = 0;
            }
            if icons::text_toggle(ui, "contrast", "Vínculo limitante", *kind == 1).clicked() {
                *kind = 1;
            }
            if icons::text_toggle(ui, "file", "Relação documental", *kind == 2).clicked() {
                *kind = 2;
            }
        });
        ui.separator();
        ui.add_space(12.0);
        let body_height = (ui.ctx().content_rect().height() - 280.0).clamp(90.0, 500.0);
        egui::ScrollArea::vertical()
            .id_salt("link-editor-scroll")
            .max_height(body_height)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                if *kind == 0 {
                    if catalog::PARAMETRIC_TYPES.len() > 1 {
                        egui::ComboBox::from_id_salt("parametric-type")
                            .selected_text(crate::i18n::tr(catalog::parametric(parametric_type).map(|t|t.info.name).unwrap_or("Escolher…")))
                            .show_ui(ui, |ui| { for ty in catalog::PARAMETRIC_TYPES { ui.selectable_value(parametric_type, ty.info.id.into(), crate::i18n::tr(ty.info.name)); } });
                        ui.add_space(12.0);
                    }
                    let ty = catalog::parametric(parametric_type).unwrap_or(&catalog::PARAMETRIC_TYPES[0]);
                    theme::section(ui, "color", "Cores participantes");
                    ui.label(RichText::new(crate::i18n::tr("Selecione duas ou mais cores para compartilhar os canais escolhidos.")).small().weak());
                    ui.add_space(8.0);
                    let mut remove = None;
                    let available: Vec<_> = p.colors().filter(|i| !participants.contains(&i.id)).collect();
                    let cell_width = ((ui.available_width() - 8.0) / 2.0).max(90.0);
                    egui::Grid::new("link-participants").num_columns(2).min_col_width(cell_width).spacing(Vec2::new(8.0, 8.0)).show(ui, |ui| {
                        for (index, id) in participants.iter().enumerate() {
                            if let Some(item) = p.item(id) {
                                participant(ui, item, cell_width, &mut remove, index);
                                if index % 2 == 1 { ui.end_row(); }
                            }
                        }
                        if participants.len() % 2 == 1 { ui.end_row(); }
                    });
                    if let Some(index) = remove { participants.remove(index); *convert = false; }
                    ui.add_space(8.0);
                    egui::ComboBox::from_id_salt("link-add-participant")
                        .selected_text(crate::i18n::tr("Adicionar cor…"))
                        .width(230.0).truncate()
                        .show_ui(ui, |ui| {
                            for item in available {
                                if ui.button(item.label()).clicked() {
                                    participants.push(item.id.clone()); *convert = false; ui.close();
                                }
                            }
                        });
                    ui.add_space(20.0);
                    theme::section(ui, "settings", "Canais compartilhados");
                    ui.add_space(8.0);
                    // Each color model has its own row: equal-width, two-line channel buttons.
                    for model in ["RGB", "HSV"] {
                        let group: Vec<_> = ty.channels.iter().filter(|c| c.model == model).collect();
                        if !group.is_empty() { channel_group(ui, model, channels, &group); ui.add_space(10.0); }
                    }
                    ui.add_space(12.0);
                    ui.label(RichText::new(crate::i18n::tr(ty.info.description)).small().weak());
                } else if *kind == 1 {
                    if ui.available_width() >= 500.0 {
                        let width = ui.available_width();
                        let left = (width * 0.30).clamp(150.0,220.0);
                        ui.horizontal_top(|ui| {
                            ui.allocate_ui_with_layout(Vec2::new(left,300.0),egui::Layout::top_down(egui::Align::Min),|ui| {
                                ui.set_width(left);
                                limiting_type(ui, limit_type, minimum);
                            });
                            ui.add_space(16.0);
                            ui.allocate_ui_with_layout(Vec2::new(width-left-24.0,300.0),egui::Layout::top_down(egui::Align::Min),|ui| {
                                ui.set_width(width-left-24.0);
                                limit_options(ui, &p, a, b, minimum, limit_type);
                            });
                        });
                    } else {
                        limiting_type(ui, limit_type, minimum);
                        ui.add_space(16.0);
                        limit_options(ui, &p, a, b, minimum, limit_type);
                    }
                } else {
                    theme::section(ui, "color", "Cores participantes");
                    color_combo(ui, &p, "link-a", a, false);
                    ui.add_space(6.0);
                    color_combo(ui, &p, "link-b", b, false);
                    ui.add_space(16.0);
                    ui.label(crate::i18n::tr("Descreve uso ou significado, sem alterar os valores."));
                    ui.add_space(8.0);
                    ui.label(crate::i18n::tr("Relação"));
                    ui.add(egui::TextEdit::singleline(relation_kind).desired_width(f32::INFINITY));
                    ui.add_space(8.0);
                    ui.label(crate::i18n::tr("Nota"));
                    ui.add(egui::TextEdit::multiline(note).desired_width(f32::INFINITY).desired_rows(3));
                }
                let ids = if *kind == 0 { participants.clone() } else { vec![a.clone(), b.clone()] };
                let native: Vec<_> = ids.iter().filter_map(|id| p.item(id)).filter(|i| !i.is_srgb()).collect();
                if *kind != 2 && !native.is_empty() {
                    ui.add_space(16.0);
                    theme::card(ui).show(ui, |ui| {
                        ui.label(RichText::new(crate::i18n::tr("Estas cores precisam de conversão explícita para sRGB.")).color(theme::accent_text(ui)));
                        for item in native {
                            ui.label(crate::i18n::tr(format!("{}: {} para sRGB {}", item.label(), item.native().space, color::hex(item.rgb))));
                            if color::preview(&item.native(), &p.profiles).is_ok_and(|rgb| !color::in_gamut(rgb)) {
                                ui.label(RichText::new(crate::i18n::tr("Fora do gamut: esta conversão limita canais e altera a aparência.")).color(theme::accent_text(ui)));
                            }
                        }
                        ui.checkbox(convert, crate::i18n::tr("Confirmo converter mantendo os IDs. Conversão e vínculo serão desfeitos juntos."));
                    });
                }
                ui.add_space(8.0);
            });
        if original_pair != (a.clone(), b.clone(), *kind) {
            *convert = false;
        }
        let ids = if *kind == 0 {
            participants.clone()
        } else {
            vec![a.clone(), b.clone()]
        };
        let needs_conversion =
            *kind != 2 && ids.iter().filter_map(|id| p.item(id)).any(|i| !i.is_srgb());
        let valid = if *kind == 0 {
            participants.len() >= 2 && channels.iter().any(|v| *v)
        } else {
            a != b && !a.is_empty() && !b.is_empty()
        };
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), 34.0),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                let apply = ui.add_enabled(
                    valid && (!needs_conversion || *convert),
                    theme::primary(if *kind == 0 {
                        "Vincular cores"
                    } else {
                        "Criar relação"
                    }),
                );
                #[cfg(test)]
                ui.ctx()
                    .data_mut(|d| d.insert_temp(egui::Id::new("test-link-apply"), apply.rect));
                if apply.clicked() {
                    let command = if *kind == 0 {
                        let ty = catalog::parametric(parametric_type)
                            .unwrap_or(&catalog::PARAMETRIC_TYPES[0]);
                        (ty.build)(
                            participants.clone(),
                            channels
                                .iter()
                                .enumerate()
                                .filter(|(_, v)| **v)
                                .map(|(c, _)| c)
                                .collect(),
                        )
                    } else if *kind == 1 {
                        let ty = catalog::limit(limit_type).unwrap_or(&catalog::LIMIT_TYPES[0]);
                        (ty.build)(a.clone(), b.clone(), *minimum)
                    } else {
                        let mut relations = p.documentary_relations.clone();
                        relations.push(Documentary {
                            id: uid(),
                            from: a.clone(),
                            to: b.clone(),
                            kind: relation_kind.clone(),
                            note: note.clone(),
                        });
                        Ok(Command::Documentation {
                            relations,
                            roles: p.roles.clone(),
                            modes: p.modes.clone(),
                        })
                    };
                    let command = match command {
                        Ok(command) => command,
                        Err(error) => {
                            self.error = Some(error);
                            return;
                        }
                    };
                    let mut commands = vec![];
                    if needs_conversion {
                        for id in &ids {
                            if let Some(item) = p.item(id).filter(|i| !i.is_srgb()) {
                                commands.push(Command::SetNative {
                                    id: id.clone(),
                                    value: NativeColor::srgb(item.rgb),
                                });
                            }
                        }
                    }
                    commands.push(command);
                    self.issue(Action::Confirm(Command::Batch(commands)));
                }
                if ui.button(crate::i18n::tr("Cancelar")).clicked() {
                    keep = false;
                }
            },
        );
        keep
    }
}

fn participant(
    ui: &mut egui::Ui,
    item: &Item,
    width: f32,
    remove: &mut Option<usize>,
    index: usize,
) {
    ui.push_id(&item.id, |ui| {
        egui::Frame::new()
            .fill(ui.visuals().faint_bg_color)
            .corner_radius(6)
            .inner_margin(8)
            .show(ui, |ui| {
                ui.set_width((width - 16.0).max(60.0));
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(24.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 4, wheel::rgb(item.rgb));
                    let label_width = (width - 16.0 - 24.0 - 28.0 - 16.0).max(10.0);
                    ui.add_sized(
                        Vec2::new(label_width, 24.0),
                        egui::Label::new(item.label())
                            .halign(egui::Align::Min)
                            .truncate(),
                    )
                    .on_hover_text(item.label());
                    if icons::button(ui, "close", "Retirar desta seleção de vínculo").clicked() {
                        *remove = Some(index);
                    }
                });
            });
    });
}

fn channel_group(
    ui: &mut egui::Ui,
    model: &str,
    channels: &mut [bool; 6],
    labels: &[&catalog::Channel],
) {
    ui.push_id(model, |ui| {
        ui.label(RichText::new(model).small().strong());
        ui.columns(3, |columns| {
            for (column, channel) in labels.iter().enumerate() {
                let catalog::Channel {
                    index,
                    letter,
                    name,
                    ..
                } = **channel;
                let ui = &mut columns[column];
                let label = format!("{}\n{}", letter, crate::i18n::tr(name));
                let response = ui.add_sized(
                    Vec2::new(ui.available_width(), 54.0),
                    egui::Button::new(label).selected(channels[index]),
                );
                if response.clicked() {
                    channels[index] = !channels[index];
                }
                #[cfg(test)]
                ui.ctx().data_mut(|d| {
                    d.insert_temp(egui::Id::new(("test-link-channel", letter)), response.rect)
                });
            }
        });
    });
}

fn limiting_type(ui: &mut egui::Ui, selected: &mut String, value: &mut f64) {
    theme::section(ui, "settings", "Tipo de limite");
    ui.add_space(8.0);
    for ty in catalog::LIMIT_TYPES {
        let width = ui.available_width();
        let response = ui.add_sized(
            Vec2::new(width, 70.0),
            egui::Button::new(crate::i18n::tr(ty.info.name)).selected(selected == ty.info.id),
        );
        if response.clicked() && selected != ty.info.id {
            *selected = ty.info.id.into();
            *value = ty.parameter.default;
        }
        response.on_hover_text(crate::i18n::tr(ty.info.description));
        ui.add_space(8.0);
    }
    if let Some(ty) = catalog::limit(selected) {
        ui.label(
            RichText::new(crate::i18n::tr(ty.info.description))
                .small()
                .weak(),
        );
    }
}

fn limit_options(
    ui: &mut egui::Ui,
    p: &Project,
    a: &mut String,
    b: &mut String,
    minimum: &mut f64,
    selected: &str,
) {
    let ty = catalog::limit(selected).unwrap_or(&catalog::LIMIT_TYPES[0]);
    theme::section(ui, ty.info.icon, ty.info.name);
    ui.add_space(10.0);
    ui.label(crate::i18n::tr("Primeira cor"));
    color_combo(ui, p, "link-a", a, false);
    ui.add_space(8.0);
    ui.label(crate::i18n::tr("Segunda cor"));
    color_combo(ui, p, "link-b", b, false);
    ui.add_space(16.0);
    ui.label(crate::i18n::tr(ty.parameter.label));
    ui.add(
        egui::Slider::new(minimum, ty.parameter.min..=ty.parameter.max)
            .clamping(egui::SliderClamping::Edits)
            .suffix(ty.parameter.suffix),
    );
    let labels: Vec<_> = ty
        .parameter
        .suggestions
        .iter()
        .map(|n| format!("{n}{}", ty.parameter.suffix))
        .collect();
    let choices: Vec<_> = ty
        .parameter
        .suggestions
        .iter()
        .zip(&labels)
        .map(|(n, l)| (*n, l.as_str()))
        .collect();
    theme::chips(ui, minimum, &choices);
    if let Some((aa, bb)) = p.item(a).zip(p.item(b)) {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(crate::i18n::tr(ty.current_label)).strong());
            ui.monospace(format!(
                "{:.4}{}",
                (ty.measure)(aa.rgb, bb.rgb),
                ty.parameter.suffix
            ));
        });
    }
    ui.add_space(12.0);
    ui.label(
        RichText::new(crate::i18n::tr(ty.explanation))
            .small()
            .weak(),
    );
}
