//! Conflict chains: snapshot labels remain readable after their project objects disappear.
use crate::{
    app::{Action, ColorCraftApp, Panel},
    conflict_log::{Entry, relation},
    i18n::tr,
    icons, theme, wheel,
};
use colorcraft_core::{
    Project,
    diagnostics::{Cause, Diagnostic, Operation, Protection},
    engine::Command,
};
use eframe::egui::{self, RichText, Vec2};

fn channel(c: usize) -> String {
    colorcraft_core::catalog::CHANNELS
        .get(c)
        .map(|c| tr(c.name))
        .unwrap_or_else(|| tr("Canais"))
}
fn title(d: &Diagnostic) -> String {
    match d.operation {
        Operation::Composition => tr("Composição em conflito"),
        Operation::CreateRelation => tr("Não foi possível criar a relação"),
        Operation::ChangeRelation => tr("Não foi possível alterar a relação"),
        Operation::Edit => {
            if let Some(c) = d.channel {
                format!(
                    "{} {}",
                    tr(if d.direction < 0 {
                        "Não foi possível reduzir"
                    } else if d.direction > 0 {
                        "Não foi possível aumentar"
                    } else {
                        "Não foi possível alterar"
                    }),
                    channel(c)
                )
            } else {
                tr("Não foi possível alterar a cor")
            }
        }
    }
}
fn reason(cause: &Cause) -> String {
    match cause {
        Cause::Protected { protection, .. } => tr(match protection {
            Protection::Manual => "Cor bloqueada",
            Protection::Direct => "Edição direta bloqueada",
            Protection::Source => "Cor vinculada a uma fonte",
            Protection::Preserved => "Valor original preservado",
        }),
        Cause::ChannelBound {
            channel: c,
            maximum,
            ..
        } => format!(
            "{} {}",
            tr(if *maximum {
                "Atingiu o máximo do canal"
            } else {
                "Atingiu o mínimo do canal"
            }),
            channel(*c)
        ),
        Cause::Gradient { .. } => tr("Cor controlada por um gradiente"),
        Cause::Contrast {
            minimum, actual, ..
        } => format!(
            "{} {:.3}:1 · {} {:.3}:1",
            tr("Contraste mínimo"),
            minimum,
            tr("Tentativa"),
            actual
        ),
        Cause::Incompatible { cycle, .. } => tr(if *cycle {
            "Ciclo de relações incompatíveis"
        } else {
            "Valores fixos ou relações incompatíveis"
        }),
        Cause::NativeSpace { .. } => tr("Conversão explícita para sRGB necessária"),
    }
}
fn record(ui: &egui::Ui, name: &str, id: &str, rect: egui::Rect) {
    #[cfg(test)]
    ui.ctx()
        .data_mut(|d| d.insert_temp(egui::Id::new((name, id)), rect));
    let _ = (ui, name, id, rect);
}
fn connector(ui: &mut egui::Ui) {
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 8.0), egui::Sense::hover());
    ui.painter().line_segment(
        [
            rect.center_top() - Vec2::new(0.0, ui.spacing().item_spacing.y),
            rect.center_bottom() + Vec2::new(0.0, ui.spacing().item_spacing.y),
        ],
        egui::Stroke::new(1.0, ui.visuals().weak_text_color()),
    );
}
fn color_node(ui: &mut egui::Ui, p: &Project, entry: &Entry, id: &str, actions: &mut Vec<Action>) {
    let Some(snapshot) = entry.colors.get(id) else {
        return;
    };
    let live = p.item(id).is_some();
    let w = ui.available_width();
    let text = if live {
        snapshot.name.clone()
    } else {
        format!(
            "{} · {}",
            snapshot.name,
            tr(if snapshot.existed {
                "Removida"
            } else {
                "Não aplicada"
            })
        )
    };
    let galley = ui.painter().layout(
        text.clone(),
        egui::FontId::proportional(13.0),
        ui.visuals().text_color(),
        (w - 38.0).max(8.0),
    );
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(w, (galley.size().y + 12.0).max(32.0)),
        if live {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        },
    );
    ui.painter().rect_filled(
        rect,
        16,
        if response.hovered() {
            ui.visuals().widgets.hovered.bg_fill
        } else {
            ui.visuals().extreme_bg_color
        },
    );
    ui.painter().rect_stroke(
        rect,
        16,
        if response.has_focus() {
            ui.visuals().selection.stroke
        } else {
            ui.visuals().widgets.noninteractive.bg_stroke
        },
        egui::StrokeKind::Inside,
    );
    ui.painter().circle_filled(
        rect.left_center() + Vec2::new(14.0, 0.0),
        8.0,
        wheel::rgb(snapshot.rgb),
    );
    ui.painter().galley(
        rect.left_center() + Vec2::new(30.0, -galley.size().y / 2.0),
        galley,
        ui.visuals().text_color(),
    );
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, live && ui.is_enabled(), &text)
    });
    record(ui, "test-conflict-color", id, rect);
    if response.clicked() && live {
        actions.push(Action::RevealColor(id.into()));
    }
    response.on_hover_text(text);
}
fn relation_node(
    ui: &mut egui::Ui,
    p: &Project,
    entry: &Entry,
    id: &str,
    c: Option<usize>,
    actions: &mut Vec<Action>,
) {
    let snapshot = entry.relations.get(id);
    let live = relation(p, id);
    let label = snapshot
        .map(|r| tr(&r.title))
        .unwrap_or_else(|| tr("Relação"));
    let label = if let Some(c) = c {
        format!("{} · {}", label, channel(c))
    } else {
        label
    };
    let label = if live.is_none() {
        format!(
            "{} · {}",
            label,
            tr(if snapshot.is_some_and(|s| s.existed) {
                "Removida"
            } else {
                "Não aplicada"
            })
        )
    } else {
        label
    };
    ui.horizontal(|ui| {
        ui.add_space(20.0);
        let width = (ui.available_width() - 20.0).max(35.0);
        let galley = ui.painter().layout(
            label.clone(),
            egui::FontId::proportional(12.0),
            ui.visuals().text_color(),
            (width - 40.0).max(8.0),
        );
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(width, (galley.size().y + 12.0).max(30.0)),
            egui::Sense::click(),
        );
        ui.painter().rect_filled(
            rect,
            15,
            if response.hovered() {
                ui.visuals().widgets.hovered.bg_fill
            } else {
                ui.visuals().faint_bg_color
            },
        );
        ui.painter().rect_stroke(
            rect,
            15,
            if response.has_focus() {
                ui.visuals().selection.stroke
            } else {
                ui.visuals().widgets.noninteractive.bg_stroke
            },
            egui::StrokeKind::Inside,
        );
        icons::paint(
            ui,
            egui::Rect::from_center_size(
                rect.left_center() + Vec2::new(16.0, 0.0),
                Vec2::splat(16.0),
            ),
            snapshot.map(|r| r.icon).unwrap_or("link"),
            ui.visuals().text_color(),
        );
        ui.painter().with_clip_rect(rect.shrink(2.0)).galley(
            rect.left_center() + Vec2::new(32.0, -galley.size().y / 2.0),
            galley,
            ui.visuals().text_color(),
        );
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Button,
                live.is_some() && ui.is_enabled(),
                &label,
            )
        });
        record(ui, "test-conflict-relation", id, rect);
        if response.clicked() && live.is_some() {
            actions.push(Action::RevealRelation(id.into()));
        }
        response.clone().on_hover_text(
            snapshot
                .map(|r| r.participants.join(" · "))
                .unwrap_or_else(|| label.clone()),
        );
        response.context_menu(|ui| {
            ui.label(RichText::new(&label).strong());
            if let Some(s) = snapshot {
                ui.add(egui::Label::new(s.participants.join(" · ")).wrap());
                if s.group {
                    ui.label(tr(
                        "A relação inteira será removida; todas as cores serão preservadas.",
                    ));
                }
            }
            ui.separator();
            let remove = ui.add_enabled(live.is_some(), egui::Button::new(tr("Desfazer vínculo")));
            record(ui, "test-conflict-remove", id, remove.rect);
            if remove.clicked() {
                actions.push(Action::Core(Command::RemoveRelation(id.into())));
                ui.close();
            }
        });
    });
}
fn chain(ui: &mut egui::Ui, p: &Project, entry: &Entry, index: usize, actions: &mut Vec<Action>) {
    let trace = &entry.diagnostic.traces[index];
    color_node(ui, p, entry, &trace.origin, actions);
    for hop in &trace.hops {
        connector(ui);
        relation_node(ui, p, entry, &hop.relation, hop.channel, actions);
        connector(ui);
        color_node(ui, p, entry, &hop.to, actions);
    }
    for id in trace.cause.relations() {
        if !trace.hops.last().is_some_and(|h| h.relation == id) {
            connector(ui);
            relation_node(ui, p, entry, &id, None, actions);
        }
    }
    connector(ui);
    egui::Frame::new()
        .inner_margin(9)
        .corner_radius(8)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .show(ui, |ui| {
            ui.set_width(ui.available_width().max(20.0));
            ui.label(RichText::new(reason(&trace.cause)).color(theme::accent_text(ui)));
            if let Cause::Incompatible { colors, .. } = &trace.cause {
                let names: Vec<_> = colors
                    .iter()
                    .filter_map(|id| entry.colors.get(id).map(|i| i.name.clone()))
                    .collect();
                if !names.is_empty() {
                    ui.label(RichText::new(names.join(" · ")).small().weak());
                }
            }
        });
}
impl ColorCraftApp {
    pub(crate) fn conflicts_panel(&mut self, ui: &mut egui::Ui) {
        let Some(doc) = self.doc() else {
            return;
        };
        let p = doc.engine.project.clone();
        let entries: Vec<_> = doc.conflicts.entries.iter().cloned().collect();
        theme::panel_title(ui, "Conflitos");
        ui.add_space(8.0);
        if ui
            .add_enabled(
                !entries.is_empty(),
                egui::Button::new(tr("Limpar histórico")),
            )
            .clicked()
        {
            if let Some(d) = self.doc_mut() {
                d.conflicts.clear();
            }
            return;
        }
        if entries.is_empty() {
            ui.add_space(18.0);
            ui.label(tr("Nenhum conflito registrado."));
            return;
        }
        let mut actions = vec![];
        let mut read = vec![];
        egui::ScrollArea::vertical()
            .id_salt(("conflict-history", &p.id))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for entry in &entries {
                    ui.push_id(entry.id, |ui| {
                        ui.add_space(10.0);
                        let width = ui.available_width();
                        let height_key = ui.id().with(("conflict-card-height", width.to_bits()));
                        if let Some(height) = ui.ctx().data(|d| d.get_temp::<f32>(height_key)) {
                            let r = egui::Rect::from_min_size(
                                ui.cursor().min,
                                Vec2::new(width, height),
                            );
                            if !r.intersects(ui.clip_rect()) {
                                ui.allocate_exact_size(r.size(), egui::Sense::hover());
                                return;
                            }
                        }
                        let card = theme::card(ui).show(ui, |ui| {
                            ui.set_width((width - 24.0).max(30.0));
                            theme::section(ui, "conflict", &title(&entry.diagnostic));
                            ui.label(
                                RichText::new(if entry.repetitions > 1 {
                                    format!(
                                        "{} — {} · ×{}",
                                        entry.first, entry.last, entry.repetitions
                                    )
                                } else {
                                    entry.first.clone()
                                })
                                .small()
                                .weak(),
                            );
                            ui.add_space(10.0);
                            if !entry.diagnostic.traces.is_empty() {
                                chain(ui, &p, entry, 0, &mut actions);
                            }
                            for index in 1..entry.diagnostic.traces.len() {
                                egui::CollapsingHeader::new(format!(
                                    "{} {}",
                                    tr("Outra cadeia"),
                                    index + 1
                                ))
                                .id_salt(index)
                                .show(ui, |ui| chain(ui, &p, entry, index, &mut actions));
                            }
                        });
                        ui.ctx()
                            .data_mut(|d| d.insert_temp(height_key, card.response.rect.height()));
                        if card.response.rect.intersects(ui.clip_rect()) {
                            read.push(entry.id);
                        }
                    });
                }
            });
        if let Some(d) = self.doc_mut() {
            for e in &mut d.conflicts.entries {
                if read.contains(&e.id) {
                    e.unread = false;
                }
            }
        }
        for action in actions {
            self.issue(action);
        }
    }
    pub(crate) fn record_engine_conflict(&mut self, continuous: bool) {
        if let Some(doc) = self.doc_mut()
            && let Some(diagnostic) = doc.engine.last_diagnostic.take()
        {
            doc.conflicts
                .record(&doc.engine.project, diagnostic, continuous);
            if self.error.is_some() {
                self.error = Some("Conflito registrado. Consulte o painel Conflitos.".into());
            }
        }
    }
    pub(crate) fn reveal_conflict_color(&mut self, id: &str) {
        let tree = self.settings.get().appearance.tree_view;
        let Some(d) = self.doc_mut() else {
            return;
        };
        let Some(item) = d.engine.project.item(id).cloned() else {
            return;
        };
        if !d.search.is_empty() && !d.visible_item_ids(tree).contains(&id.to_owned()) {
            d.search.clear();
        }
        d.folder = if tree { None } else { item.parent_id.clone() };
        let mut parent = item.parent_id.clone();
        while let Some(id) = parent {
            d.expanded.insert(id.clone());
            parent = d.engine.project.item(&id).and_then(|i| i.parent_id.clone());
        }
        d.selected = vec![id.into()];
        d.active = Some(id.into());
        d.reveal_item = Some(id.into());
        d.focused_relation = None;
        self.issue(Action::ShowPanel(Panel::Palette));
    }
    pub(crate) fn reveal_conflict_relation(&mut self, id: &str) {
        if let Some(d) = self.doc_mut()
            && relation(&d.engine.project, id).is_some()
        {
            d.focused_relation = Some(id.into());
            d.scroll_relation = Some(id.into());
            self.issue(Action::ShowPanel(Panel::Links));
        }
    }
}
