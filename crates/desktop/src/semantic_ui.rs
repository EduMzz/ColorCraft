//! Semantic roles and project modes. Empty bindings are intentional, never inferred.
use crate::{
    app::{Action, ColorCraftApp},
    i18n::tr,
    icons, theme, wheel,
};
use colorcraft_core::{engine::Command, *};
use eframe::egui::{self, RichText, Vec2};

fn documentation(p: &Project, roles: Vec<Role>, modes: Vec<Mode>) -> Command {
    Command::Documentation {
        relations: p.documentary_relations.clone(),
        roles,
        modes,
    }
}

/// Bindings accept exactly one color. Dropping folders or multiple colors is rejected.
pub(crate) fn dropped_color(p: &Project, ids: &[String]) -> Option<String> {
    (ids.len() == 1)
        .then(|| p.item(&ids[0]))
        .flatten()
        .filter(|i| i.is_color())
        .map(|i| i.id.clone())
}

fn binding(ui: &mut egui::Ui, p: &Project, key: &str, value: &mut String) -> bool {
    let mut changed = false;
    ui.push_id(key, |ui| {
        let width = ui.available_width();
        let frame = egui::Frame::new()
            .fill(ui.visuals().extreme_bg_color)
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .corner_radius(6)
            .inner_margin(7);
        let result = frame.show(ui, |ui| {
            ui.set_width((width - 14.0).max(40.0));
            ui.horizontal(|ui| {
                if let Some(item) = p.item(value) {
                    let (r, _) = ui.allocate_exact_size(Vec2::splat(22.0), egui::Sense::hover());
                    ui.painter().rect_filled(r, 4, wheel::rgb(item.rgb));
                } else {
                    let (r, _) = ui.allocate_exact_size(Vec2::splat(22.0), egui::Sense::hover());
                    icons::paint(ui, r.shrink(2.0), "dropper", ui.visuals().weak_text_color());
                }
                egui::ComboBox::from_id_salt("binding")
                    .width((ui.available_width() - 38.0).max(35.0))
                    .truncate()
                    .selected_text(
                        p.item(value)
                            .map(Item::label)
                            .unwrap_or_else(|| tr("Sem associação")),
                    )
                    .show_ui(ui, |ui| {
                        for color in p.colors() {
                            changed |= ui
                                .selectable_value(value, color.id.clone(), color.label())
                                .changed();
                        }
                    });
                if !value.is_empty() && icons::button(ui, "unlink", "Remover associação").clicked()
                {
                    value.clear();
                    changed = true;
                }
            });
        });
        let response = ui.interact(
            result.response.rect,
            ui.id().with("color-drop"),
            egui::Sense::hover(),
        );
        if let Some(payload) = response.dnd_hover_payload::<Vec<String>>() {
            let accepted = dropped_color(p, &payload).is_some();
            ui.painter().rect_stroke(
                response.rect,
                6,
                egui::Stroke::new(
                    2.0,
                    if accepted {
                        ui.visuals().selection.stroke.color
                    } else {
                        ui.visuals().error_fg_color
                    },
                ),
                egui::StrokeKind::Inside,
            );
        }
        if let Some(payload) = response.dnd_release_payload::<Vec<String>>()
            && let Some(color) = dropped_color(p, &payload)
        {
            *value = color;
            changed = true;
        }
        response.on_hover_text(tr("Arraste uma cor da paleta ou escolha na lista"));
        #[cfg(test)]
        ui.ctx().data_mut(|d| {
            d.insert_temp(
                egui::Id::new(("test-role-binding", key)),
                result.response.rect,
            )
        });
    });
    changed
}

impl ColorCraftApp {
    pub(crate) fn roles_panel(&mut self, ui: &mut egui::Ui) {
        let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
        let Some(p) = self.project().cloned() else {
            return;
        };
        let mut roles = p.roles.clone();
        let mut modes = p.modes.clone();
        let mut changed = false;
        let mut finish = false;
        let mut structural = false;
        theme::panel_title(ui, "Papéis semânticos");
        ui.add_space(8.0);
        if icons::text_button(ui, "plus", "Novo papel").clicked() {
            let mut index = roles.len() + 1;
            while roles.iter().any(|r| r.name == format!("role.{index}")) {
                index += 1;
            }
            roles.push(Role {
                id: uid(),
                name: format!("role.{index}"),
                color_id: String::new(),
                description: String::new(),
            });
            changed = true;
            structural = true;
        }
        if roles.is_empty() {
            ui.add_space(18.0);
            ui.label(tr(
                "Crie um papel, como texto.principal, e arraste uma cor até sua associação.",
            ));
        }
        let mut remove = None;
        for (index, role) in roles.iter_mut().enumerate() {
            ui.add_space(10.0);
            let width = ui.available_width();
            theme::card(ui).show(ui, |ui| {
                ui.set_width((width - 24.0).max(50.0));
                ui.push_id(&role.id, |ui| {
                    ui.horizontal(|ui| {
                        let r = ui.add(
                            egui::TextEdit::singleline(&mut role.name)
                                .desired_width((ui.available_width() - 38.0).max(40.0))
                                .hint_text("role.name"),
                        );
                        changed |= r.changed();
                        finish |= r.lost_focus();
                        if icons::button(ui, "trash", "Excluir papel").clicked() {
                            remove = Some(index);
                        }
                    });
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut role.description)
                            .desired_width(f32::INFINITY)
                            .hint_text(tr("Descrição de uso")),
                    );
                    changed |= r.changed();
                    finish |= r.lost_focus();
                    ui.add_space(6.0);
                    ui.label(RichText::new(tr("Sem modo · padrão")).small().strong());
                    let result =
                        binding(ui, &p, &format!("{}:default", role.id), &mut role.color_id);
                    changed |= result;
                    structural |= result;
                    let mut remove_binding = None;
                    for mode in &mut modes {
                        if let Some(value) = mode.overrides.get_mut(&role.id) {
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&mode.name).small().strong());
                                if icons::button(ui, "close", "Remover modo deste papel").clicked()
                                {
                                    remove_binding = Some(mode.id.clone());
                                }
                            });
                            let result =
                                binding(ui, &p, &format!("{}:{}", role.id, mode.id), value);
                            changed |= result;
                            structural |= result;
                        }
                    }
                    if let Some(id) = remove_binding {
                        if let Some(mode) = modes.iter_mut().find(|m| m.id == id) {
                            mode.overrides.remove(&role.id);
                        }
                        changed = true;
                        structural = true;
                    }
                    ui.add_space(7.0);
                    let add_mode = ui.button(tr("Adicionar modo…"));
                    #[cfg(test)]
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(
                            egui::Id::new(("test-add-role-mode", &role.id)),
                            add_mode.rect,
                        )
                    });
                    egui::Popup::menu(&add_mode)
                        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                        .show(|ui| {
                            for mode in modes
                                .iter_mut()
                                .filter(|m| !m.overrides.contains_key(&role.id))
                            {
                                if ui.button(&mode.name).clicked() {
                                    mode.overrides.insert(role.id.clone(), String::new());
                                    changed = true;
                                    structural = true;
                                    ui.close();
                                }
                            }
                            ui.separator();
                            let key = ui.id().with(("new-role-mode", &p.id, &role.id));
                            let mut name = ui
                                .ctx()
                                .data(|d| d.get_temp::<String>(key))
                                .unwrap_or_default();
                            let r = ui.add(
                                egui::TextEdit::singleline(&mut name)
                                    .hint_text(tr("Nome do novo modo")),
                            );
                            #[cfg(test)]
                            ui.ctx().data_mut(|d| {
                                d.insert_temp(egui::Id::new("test-new-role-mode"), r.rect)
                            });
                            let enter = (r.has_focus() || r.lost_focus()) && enter_pressed;
                            if (enter || icons::text_button(ui, "plus", "Criar modo").clicked())
                                && !name.trim().is_empty()
                            {
                                let clean = name.trim();
                                if let Some(mode) = modes.iter_mut().find(|m| m.name == clean) {
                                    mode.overrides.entry(role.id.clone()).or_default();
                                } else {
                                    let mut mode = Mode {
                                        id: uid(),
                                        name: clean.into(),
                                        ..Default::default()
                                    };
                                    mode.overrides.insert(role.id.clone(), String::new());
                                    modes.push(mode);
                                }
                                changed = true;
                                structural = true;
                                name.clear();
                                ui.close();
                            }
                            ui.ctx().data_mut(|d| d.insert_temp(key, name));
                        });
                });
            });
        }
        if let Some(index) = remove {
            let role = roles.remove(index);
            for mode in &mut modes {
                mode.overrides.remove(&role.id);
            }
            changed = true;
            structural = true;
        }
        if changed && roles.iter().all(|r| !r.name.trim().is_empty()) {
            let command = documentation(&p, roles, modes);
            self.issue(if structural {
                Action::Core(command)
            } else {
                Action::Preview(command)
            });
        }
        if finish {
            self.issue(Action::Finish);
        }
    }

    pub(crate) fn modes_editor(&mut self, ui: &mut egui::Ui, p: &Project) {
        let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
        let mut modes = p.modes.clone();
        let mut changed = false;
        let mut structural = false;
        let mut finish = false;
        let width = ui.available_width();
        theme::card(ui).show(ui, |ui| {
            ui.set_width((width - 24.0).max(40.0));
            theme::section(ui, "modes", "Modos da paleta");
            ui.label(
                RichText::new(tr(
                    "Crie os modos aqui e associe suas cores no painel Papéis semânticos.",
                ))
                .small()
                .weak(),
            );
            let mut remove = None;
            for (index, mode) in modes.iter_mut().enumerate() {
                ui.push_id(&mode.id, |ui| {
                    ui.horizontal(|ui| {
                        let r = ui.add(
                            egui::TextEdit::singleline(&mut mode.name)
                                .desired_width((ui.available_width() - 38.0).max(40.0)),
                        );
                        changed |= r.changed();
                        finish |= r.lost_focus();
                        if icons::button(ui, "trash", "Excluir modo e suas associações").clicked()
                        {
                            remove = Some(index);
                        }
                    })
                });
            }
            if let Some(index) = remove {
                modes.remove(index);
                changed = true;
                structural = true;
            }
            let key = ui.id().with(("new-project-mode", &p.id));
            let mut name = ui
                .ctx()
                .data(|d| d.get_temp::<String>(key))
                .unwrap_or_default();
            ui.horizontal(|ui| {
                let r = ui.add(
                    egui::TextEdit::singleline(&mut name)
                        .desired_width((ui.available_width() - 38.0).max(40.0))
                        .hint_text(tr("Nome do novo modo")),
                );
                let add = icons::button(ui, "plus", "Criar modo").clicked()
                    || ((r.has_focus() || r.lost_focus()) && enter_pressed);
                if add && !name.trim().is_empty() && !modes.iter().any(|m| m.name == name.trim()) {
                    modes.push(Mode {
                        id: uid(),
                        name: name.trim().into(),
                        ..Default::default()
                    });
                    name.clear();
                    changed = true;
                    structural = true;
                }
            });
            ui.ctx().data_mut(|d| d.insert_temp(key, name));
            if icons::text_button(ui, "roles", "Abrir papéis semânticos").clicked() {
                self.issue(Action::ShowPanel(crate::app::Panel::Roles));
            }
        });
        if changed && modes.iter().all(|m| !m.name.trim().is_empty()) {
            let command = documentation(p, p.roles.clone(), modes);
            self.issue(if structural {
                Action::Core(command)
            } else {
                Action::Preview(command)
            });
        }
        if finish {
            self.issue(Action::Finish);
        }
    }
}
