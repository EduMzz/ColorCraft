use crate::{
    app::{ACCENT, Action, ColorCraftApp, Panel},
    dialogs::Dialog,
    wheel,
};
use colorcraft_core::{color, engine::Command, *};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2};

fn parse_component_input(text: &str, current: f64, hue: bool) -> Option<f64> {
    let text = text.trim().replace(',', ".");
    if text.is_empty() {
        return None;
    }
    let (body, degrees) = if let Some(v) = text.strip_suffix('°').or_else(|| text.strip_suffix('º'))
    {
        (v.trim(), true)
    } else if text.to_ascii_lowercase().ends_with("deg") {
        (text[..text.len().checked_sub(3)?].trim(), true)
    } else {
        (text.as_str(), false)
    };
    if degrees && !hue {
        return None;
    }
    let (operator, operand_text) = if let Some(v) = body.strip_prefix('+') {
        ('+', v)
    } else if let Some(v) = body.strip_prefix('-') {
        ('-', v)
    } else if let Some(v) = body.strip_prefix('*') {
        ('*', v)
    } else if let Some(v) = body.strip_prefix('/') {
        ('/', v)
    } else {
        ('=', body)
    };
    let percent = body.ends_with('%');
    let operand = operand_text
        .strip_suffix('%')
        .unwrap_or(operand_text)
        .trim()
        .parse::<f64>()
        .ok()?;
    if !operand.is_finite() {
        return None;
    }
    let relative = |amount: f64, increase: bool| {
        let delta = if percent {
            current * amount / 100.0
        } else {
            amount
        };
        if increase {
            current + delta
        } else {
            current - delta
        }
    };
    let result = match operator {
        '+' => relative(operand, true),
        '-' => relative(operand, false),
        '*' => current * if percent { operand / 100.0 } else { operand },
        '/' => {
            let divisor = operand;
            if divisor == 0.0 {
                return None;
            }
            current / divisor
        }
        _ if percent => current * (1.0 - operand / 100.0),
        _ => operand,
    };
    result
        .is_finite()
        .then_some(if hue { color::wrap(result) } else { result })
}

fn is_hue_component(mode: &str, index: usize) -> bool {
    (index == 0 && matches!(mode, "HSV" | "HSL")) || (index == 2 && mode == "OKLCH")
}

fn component_value(value: f64, mode: &str) -> String {
    let decimals = if mode == "OKLCH" { 5 } else { 2 };
    format!("{value:.decimals$}")
}

fn technical_summary(p: &Project, id: &str) -> Option<String> {
    let counts = [
        (
            p.links.iter().filter(|l| l.a == id || l.b == id).count(),
            "vínculo de canais",
            "vínculos de canais",
        ),
        (
            p.harmonies
                .iter()
                .filter(|h| h.ids.iter().any(|i| i == id))
                .count(),
            "harmonia",
            "harmonias",
        ),
        (
            p.gradients
                .iter()
                .filter(|g| {
                    g.a == id
                        || g.b == id
                        || (g.ids.iter().any(|i| i == id) && !g.detached.iter().any(|i| i == id))
                })
                .count(),
            "gradiente",
            "gradientes",
        ),
        (
            p.limits.iter().filter(|l| l.a == id || l.b == id).count(),
            "limite de contraste",
            "limites de contraste",
        ),
    ];
    let parts: Vec<_> = counts
        .into_iter()
        .filter(|(n, _, _)| *n > 0)
        .map(|(n, singular, plural)| {
            let label = if n == 1 { singular } else { plural };
            crate::i18n::tr(format!("{n} {label}"))
        })
        .collect();
    (!parts.is_empty())
        .then(|| crate::i18n::tr(format!("Relações técnicas: {}", parts.join(" · "))))
}

fn technical_badge(ui: &egui::Ui, p: &Project, item: &Item, rect: Rect, ink: Color32) {
    let Some(summary) = technical_summary(p, &item.id) else {
        return;
    };
    crate::icons::paint(ui, rect.shrink(3.0), "link", ink);
    let response = ui.interact(
        rect,
        ui.id().with(("technical-relations", &item.id)),
        Sense::hover(),
    );
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Label, ui.is_enabled(), &summary)
    });
    response.on_hover_text(summary);
    #[cfg(test)]
    ui.ctx()
        .data_mut(|d| d.insert_temp(egui::Id::new(("test-technical-badge", &item.id)), rect));
    if p.gradients
        .iter()
        .any(|g| g.ids.contains(&item.id) && !g.detached.contains(&item.id))
    {
        let dependency = rect.translate(Vec2::new(24.0, 0.0));
        crate::icons::paint(ui, dependency.shrink(3.0), "gradient", ink);
        ui.interact(dependency, ui.id().with(("gradient-dependency", &item.id)), Sense::hover())
            .on_hover_text(crate::i18n::tr("Intermediária dependente de gradiente. Destaque-a no painel Vínculos para editar livremente."));
    }
}

/// The viewport fills the panel. Padding belongs to its contents, never outside its scrollbar.
fn panel_scroll(ui: &mut egui::Ui, id: &str, top: i8, body: impl FnOnce(&mut egui::Ui)) {
    let width = ui
        .available_width()
        .min(ui.clip_rect().right() - ui.available_rect_before_wrap().left())
        .max(1.0);
    let output = egui::ScrollArea::vertical()
        .id_salt(id)
        .max_width(width)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Measure height separately: an unwrapped field or long label must never
            // expand the viewport horizontally and push its scrollbar past the panel.
            let bounds = ui.available_rect_before_wrap();
            let mut content = ui.new_child(
                egui::UiBuilder::new()
                    .id(ui.id().with("padded-content"))
                    .max_rect(bounds),
            );
            let mut clip = ui.clip_rect();
            clip.max.x = clip.max.x.min(bounds.right());
            content.set_clip_rect(clip);
            egui::Frame::new()
                .inner_margin(egui::Margin {
                    left: 14,
                    right: 14,
                    top,
                    bottom: 14,
                })
                .show(&mut content, |ui| {
                    ui.set_min_width(ui.available_width());
                    body(ui);
                });
            ui.allocate_space(Vec2::new(bounds.width(), content.min_rect().height()));
        });
    #[cfg(test)]
    ui.ctx().data_mut(|d| {
        d.insert_temp(
            egui::Id::new(("test-scroll-viewport", id)),
            output.inner_rect,
        );
        d.insert_temp(
            egui::Id::new(("test-scroll-content-size", id)),
            output.content_size,
        );
    });
    #[cfg(not(test))]
    let _ = output;
}

impl ColorCraftApp {
    fn toggle_item_lock(&mut self, p: &Project, item: &Item) {
        if item.source_locked {
            if let Some((source, capture)) = p.capture_for_color(&item.id) {
                self.dialog = Some(Dialog::UnlinkCapture {
                    item_id: item.id.clone(),
                    source_id: source.id.clone(),
                    capture_id: capture.id.clone(),
                });
            }
        } else {
            self.issue(Action::Core(Command::Lock {
                id: item.id.clone(),
                locked: !item.locked,
            }));
        }
    }
    fn direct_lock_control(&mut self, ui: &mut egui::Ui, item: &Item, rect: Rect, ink: Color32) {
        #[cfg(test)]
        ui.ctx()
            .data_mut(|d| d.insert_temp(egui::Id::new(("test-direct-lock", &item.id)), rect));
        if crate::icons::at_ink(
            ui,
            rect,
            if item.direct_locked {
                "hand-blocked"
            } else {
                "hand"
            },
            if item.direct_locked {
                "Liberar edição direta · relações continuam ativas"
            } else {
                "Bloquear edição direta · permitir alterações pelas relações"
            },
            item.direct_locked,
            if item.direct_locked {
                ui.visuals().selection.stroke.color
            } else {
                ink
            },
        )
        .clicked()
        {
            self.issue(Action::Core(Command::DirectLock {
                id: item.id.clone(),
                locked: !item.direct_locked,
            }));
        }
    }
    pub fn panel(&mut self, ui: &mut egui::Ui, panel: Panel) {
        #[cfg(test)]
        ui.ctx().data_mut(|d| {
            d.insert_temp(
                egui::Id::new(("test-panel-body", panel)),
                ui.available_rect_before_wrap(),
            )
        });
        match panel {
            Panel::Palette => self.palette_panel(ui),
            Panel::Color => panel_scroll(ui, "color-scroll", 14, |ui| self.color_panel(ui)),
            Panel::Links => panel_scroll(ui, "links-scroll", 14, |ui| self.links_panel(ui)),
            Panel::Details => panel_scroll(ui, "details-scroll", 14, |ui| self.details_panel(ui)),
            Panel::Sources => self.source_panel(ui),
            Panel::Conflicts => {
                egui::Frame::new()
                    .inner_margin(14)
                    .show(ui, |ui| self.conflicts_panel(ui));
            }
            Panel::Roles => panel_scroll(ui, "roles-scroll", 14, |ui| self.roles_panel(ui)),
        }
    }
    fn palette_panel(&mut self, ui: &mut egui::Ui) {
        let Some(doc) = self.doc() else { return };
        let p = doc.engine.project.clone();
        let folder = doc.folder.clone();
        let selected = doc.selected.clone();
        let mut search = doc.search.clone();
        let mut tree = self.settings.get().appearance.tree_view;
        let level =
            crate::presentation::CardLevel::from_zoom(self.settings.get().appearance.card_zoom);
        egui::Frame::new()
            .inner_margin(egui::Margin {
                left: 14,
                right: 14,
                top: 14,
                bottom: 0,
            })
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                self.editable_title(ui, &p.name, None, 18.0);
                ui.label(
                    RichText::new(crate::i18n::tr(format!(
                        "{} cores · {} pastas",
                        p.count(),
                        p.items.iter().filter(|i| i.kind == "folder").count()
                    )))
                    .small()
                    .weak(),
                );
                ui.add_space(12.0);
                let narrow = ui.available_width() < 420.0;
                ui.horizontal_wrapped(|ui| {
                    if crate::icons::toggle(ui, "grid", "Visualização em cards", !tree).clicked()
                    {
                        tree = false
                    }
                    if crate::icons::toggle(ui, "tree", "Visualização em árvore", tree).clicked()
                    {
                        tree = true
                    }
                    if !narrow {
                        ui.separator();
                        ui.add(
                            egui::TextEdit::singleline(&mut search)
                                .hint_text(crate::i18n::tr("Buscar cores, pastas ou tags"))
                                .desired_width((ui.available_width() - 260.0).clamp(130.0, 280.0)),
                        );
                    }
                    if crate::icons::enabled_button(
                        ui,
                        self.picking.is_none(),
                        "color",
                        "Adicionar cor",
                    )
                    .clicked()
                    {
                        self.issue(Action::Core(Command::Add(Item::new(
                            "color",
                            self.insertion_parent(),
                        ))));
                    }
                    if crate::icons::enabled_button(
                        ui,
                        self.picking.is_none(),
                        "folder",
                        "Adicionar pasta",
                    )
                    .clicked()
                    {
                        self.issue(Action::Core(Command::Add(Item::new(
                            "folder",
                            self.insertion_parent(),
                        ))));
                    }
                });
                if narrow {
                    ui.add(
                        egui::TextEdit::singleline(&mut search)
                            .hint_text(crate::i18n::tr("Buscar cores, pastas ou tags"))
                            .desired_width(ui.available_width()),
                    );
                }
                ui.horizontal(|ui| {
                    let root_icon =
                        crate::icons::button(ui, "home", "Raiz da paleta · solte itens para mover");
                    if root_icon.clicked() {
                        self.issue(Action::Folder(None));
                    }
                    if let Some(ids) = root_icon.dnd_release_payload::<Vec<String>>() {
                        self.issue(Action::Core(Command::Move {
                            ids: (*ids).clone(),
                            parent: None,
                        }));
                    }
                    let root = ui.small_button(crate::i18n::tr("Paleta"));
                    if root.clicked() {
                        self.issue(Action::Folder(None))
                    }
                    if let Some(ids) = root.dnd_release_payload::<Vec<String>>() {
                        self.issue(Action::Core(Command::Move {
                            ids: (*ids).clone(),
                            parent: None,
                        }));
                    }
                    let mut crumbs = vec![];
                    let mut next = folder.as_deref();
                    for _ in 0..100 {
                        let Some(id) = next else { break };
                        let Some(i) = p.item(id) else { break };
                        crumbs.push(i.clone());
                        next = i.parent_id.as_deref();
                    }
                    for i in crumbs.into_iter().rev() {
                        ui.label(crate::i18n::tr("/"));
                        let r = ui.small_button(i.label());
                        if r.clicked() {
                            self.issue(Action::Folder(Some(i.id.clone())))
                        }
                        if let Some(ids) = r.dnd_release_payload::<Vec<String>>() {
                            self.issue(Action::Core(Command::Move {
                                ids: (*ids).clone(),
                                parent: Some(i.id),
                            }));
                        }
                    }
                    if !tree {
                        let available = ui.available_width();
                        if available > 150.0 {
                            ui.add_space(available - 146.0);
                        }
                        let mut level = crate::presentation::CardLevel::from_zoom(
                            self.settings.get().appearance.card_zoom,
                        );
                        egui::ComboBox::from_id_salt("card-presentation-level")
                            .selected_text(level.short_label())
                            .width(132.0)
                            .show_ui(ui, |ui| {
                                for choice in crate::presentation::CardLevel::ALL {
                                    ui.selectable_value(&mut level, choice, choice.label());
                                }
                            });
                        if level
                            != crate::presentation::CardLevel::from_zoom(
                                self.settings.get().appearance.card_zoom,
                            )
                        {
                            self.settings
                                .update(|prefs| prefs.appearance.card_zoom = level.zoom());
                        }
                    }
                });
                if let Some(d) = self.doc_mut() {
                    d.search = search.clone();
                }
                if tree != self.settings.get().appearance.tree_view {
                    self.settings.update(|s| {
                        s.appearance.tree_view = tree;
                    });
                }
                if !selected.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            RichText::new(crate::i18n::tr(format!(
                                "{} selecionado(s)",
                                selected.len()
                            )))
                            .small()
                            .color(crate::theme::accent_text(ui)),
                        );
                        if crate::icons::enabled_button(
                            ui,
                            self.picking.is_none(),
                            "spark",
                            "Criar harmonia ou gradiente",
                        )
                        .clicked()
                        {
                            self.open_composition();
                        }
                        if crate::icons::enabled_button(
                            ui,
                            self.picking.is_none(),
                            "link",
                            "Criar vínculo, limite ou relação documental",
                        )
                        .clicked()
                        {
                            self.open_link();
                        }
                        if crate::icons::enabled_button(
                            ui,
                            self.picking.is_none(),
                            "move",
                            "Mover seleção",
                        )
                        .clicked()
                        {
                            self.issue(Action::Show(Dialog::Move {
                                ids: selected.clone(),
                                parent: folder.clone(),
                            }));
                        }
                        if crate::icons::enabled_button(
                            ui,
                            self.picking.is_none(),
                            "close",
                            "Limpar seleção",
                        )
                        .clicked()
                            && let Some(d) = self.doc_mut()
                        {
                            d.selected.clear();
                            d.active = None;
                        }
                    });
                } else {
                    ui.label(
                        RichText::new(crate::i18n::tr(
                            "Clique para editar · Ctrl para selecionar · arraste para organizar",
                        ))
                        .small()
                        .weak(),
                    );
                }
                ui.separator();
                ui.add_space(9.0);
            });
        let query = search.to_lowercase();
        let visible: Vec<Item> = p
            .items
            .iter()
            .filter(|i| {
                if query.is_empty() {
                    i.parent_id == folder
                } else {
                    i.name.to_lowercase().contains(&query)
                        || i.css_name.to_lowercase().contains(&query)
                        || i.tags.iter().any(|s| s.to_lowercase().contains(&query))
                }
            })
            .cloned()
            .collect();
        panel_scroll(ui, "palette-scroll", 0, |ui| {
            if visible.is_empty() {
                ui.add_space(60.0);
                ui.label(
                    RichText::new(crate::i18n::tr(if query.is_empty() {
                        "Uma página em branco para suas próximas combinações."
                    } else {
                        "Nenhum item corresponde à busca."
                    }))
                    .size(18.0)
                    .weak(),
                );
                if query.is_empty() && ui.button(crate::i18n::tr("+ Criar uma cor")).clicked() {
                    self.issue(Action::Core(Command::Add(Item::new(
                        "color",
                        self.insertion_parent(),
                    ))));
                }
            }
            if tree {
                for i in &visible {
                    self.tree_item(ui, &p, i, 0, &selected, query.is_empty());
                }
            } else {
                let width = level.size().x;
                let gap = level.gap();
                let cols = ((ui.available_width() + gap) / (width + gap))
                    .floor()
                    .max(1.0) as usize;
                let actual = ((ui.available_width() - (cols - 1) as f32 * gap) / cols as f32)
                    .min(width + 24.0);
                egui::Grid::new("palette-cards")
                    .min_col_width(0.0)
                    .spacing([gap, gap])
                    .show(ui, |ui| {
                        for (index, item) in visible.iter().enumerate() {
                            self.card(
                                ui,
                                &p,
                                item,
                                Vec2::new(actual.min(width), level.size().y),
                                selected.contains(&item.id),
                                level,
                            );
                            if (index + 1) % cols == 0 {
                                ui.end_row();
                            }
                        }
                    });
            }
        });
    }
    fn card(
        &mut self,
        ui: &mut egui::Ui,
        p: &Project,
        item: &Item,
        size: Vec2,
        selected: bool,
        level: crate::presentation::CardLevel,
    ) {
        let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());
        #[cfg(test)]
        ui.ctx()
            .data_mut(|d| d.insert_temp(egui::Id::new(("test-card", &item.id)), rect));
        if level.swatch_only() {
            if item.is_color() {
                ui.painter().rect_filled(rect, 3, wheel::rgb(item.rgb));
            } else {
                crate::icons::paint(
                    ui,
                    rect.shrink(1.0),
                    "folder",
                    ui.visuals().weak_text_color(),
                );
            }
            if item.is_color() {
                ui.painter().rect_stroke(
                    rect,
                    3,
                    Stroke::new(0.7, ui.visuals().widgets.noninteractive.bg_stroke.color),
                    egui::StrokeKind::Outside,
                );
            }
            if selected || response.hovered() || response.has_focus() {
                ui.painter().rect_stroke(
                    rect.expand(1.0),
                    3,
                    Stroke::new(
                        if selected { 2.0 } else { 1.0 },
                        if selected {
                            ACCENT
                        } else {
                            ui.visuals().text_color()
                        },
                    ),
                    egui::StrokeKind::Outside,
                );
            }
            self.item_response(ui, p, item, response);
            return;
        }
        let zoom: f32 = if level == crate::presentation::CardLevel::Summary {
            0.8
        } else {
            1.0
        };
        let painter = ui.painter();
        let radius = 8.0;
        painter.rect_filled(rect, radius, ui.visuals().faint_bg_color);
        let top = Rect::from_min_size(rect.min, Vec2::new(rect.width(), size.y * 0.67));
        if item.is_color() {
            painter.rect_filled(top, radius, wheel::rgb(item.rgb));
            let fg = if color::luminance(item.rgb) > 0.179 {
                Color32::from_rgb(20, 22, 26)
            } else {
                Color32::WHITE
            };
            painter.text(
                top.min + Vec2::new(12.0, 15.0),
                Align2::LEFT_TOP,
                color::hex(item.rgb),
                FontId::monospace(10.0 * zoom.max(0.9)),
                fg,
            );
            if selected {
                painter.circle_filled(
                    top.max - Vec2::new(17.0, 17.0),
                    9.0,
                    Color32::from_black_alpha(180),
                );
                crate::widgets::check(ui, top.max - Vec2::new(17.0, 17.0));
            }
            technical_badge(
                ui,
                p,
                item,
                Rect::from_min_size(top.left_bottom() + Vec2::new(8.0, -30.0), Vec2::splat(24.0)),
                fg,
            );
        } else {
            crate::widgets::folder(ui, top);
            let colors: Vec<_> = p
                .colors()
                .filter(|c| c.parent_id.as_deref() == Some(item.id.as_str()))
                .take(6)
                .collect();
            let w = colors.len() as f32 * 19.0;
            for (i, c) in colors.iter().enumerate() {
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(
                            top.center().x - w / 2.0 + i as f32 * 19.0,
                            top.bottom() - 24.0,
                        ),
                        Vec2::new(16.0, 6.0),
                    ),
                    2.0,
                    wheel::rgb(c.rgb),
                );
            }
        }
        let label = item.label();
        let maxchars = (rect.width() / 7.5).max(8.0) as usize;
        let label = if label.chars().count() > maxchars {
            format!(
                "{}…",
                label
                    .chars()
                    .take(maxchars.saturating_sub(1))
                    .collect::<String>()
            )
        } else {
            label
        };
        painter.text(
            Pos2::new(rect.left() + 11.0, top.bottom() + 16.0),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(13.0 * zoom.clamp(0.85, 1.15)),
            ui.visuals().text_color(),
        );
        if level == crate::presentation::CardLevel::Full {
            let info = if item.is_color() {
                if item.is_srgb() {
                    format!(
                        "RGB  {:.0} · {:.0} · {:.0}",
                        item.rgb[0], item.rgb[1], item.rgb[2]
                    )
                } else {
                    format!("{} · prévia sRGB", item.native().space.to_uppercase())
                }
            } else {
                format!(
                    "{} itens",
                    p.items
                        .iter()
                        .filter(|i| i.parent_id.as_deref() == Some(item.id.as_str()))
                        .count()
                )
            };
            painter.text(
                Pos2::new(rect.left() + 11.0, rect.bottom() - 15.0),
                Align2::LEFT_CENTER,
                info,
                FontId::monospace(9.0),
                ui.visuals().weak_text_color(),
            );
        }
        painter.rect_stroke(
            rect,
            radius,
            Stroke::new(
                if selected { 2.0 } else { 1.0 },
                if selected {
                    ACCENT
                } else if response.hovered() {
                    ui.visuals().text_color()
                } else {
                    ui.visuals().widgets.noninteractive.bg_stroke.color
                },
            ),
            egui::StrokeKind::Inside,
        );
        let lock_rect =
            Rect::from_min_size(top.right_top() + Vec2::new(-34.0, 5.0), Vec2::splat(28.0));
        let direct_rect = lock_rect.translate(Vec2::new(-30.0, 0.0));
        #[cfg(test)]
        ui.ctx()
            .data_mut(|d| d.insert_temp(egui::Id::new(("test-card-lock", &item.id)), lock_rect));
        let mut over_control = false;
        if item.is_color() && level.controls() && self.picking.is_none() {
            over_control = ui.input(|i| {
                i.pointer
                    .hover_pos()
                    .is_some_and(|p| lock_rect.contains(p) || direct_rect.contains(p))
            });
            let ink = if color::luminance(item.rgb) > 0.179 {
                Color32::from_rgb(24, 25, 29)
            } else {
                Color32::WHITE
            };
            if crate::icons::at_ink(
                ui,
                lock_rect,
                if item.locked || item.source_locked {
                    "lock"
                } else {
                    "unlock"
                },
                if item.source_locked {
                    "Desvincular captura e desbloquear cor"
                } else if item.locked {
                    "Desbloquear cor"
                } else {
                    "Bloquear cor"
                },
                false,
                ink,
            )
            .clicked()
            {
                self.toggle_item_lock(p, item);
            }
            self.direct_lock_control(ui, item, direct_rect, ink);
        }
        if !over_control {
            self.item_response(ui, p, item, response);
        }
    }
    fn tree_item(
        &mut self,
        ui: &mut egui::Ui,
        p: &Project,
        item: &Item,
        depth: usize,
        selected: &[String],
        recurse: bool,
    ) {
        if depth > 100 {
            return;
        }
        let open = self.doc().is_some_and(|d| d.expanded.contains(&item.id));
        let is_selected = selected.contains(&item.id);
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), 36.0),
            Sense::click_and_drag(),
        );
        #[cfg(test)]
        ui.ctx()
            .data_mut(|d| d.insert_temp(egui::Id::new(("test-row", &item.id)), rect));
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::SelectableLabel,
                ui.is_enabled(),
                is_selected,
                item.label(),
            )
        });
        if is_selected || response.hovered() || response.has_focus() {
            ui.painter().rect_filled(
                rect,
                4,
                if is_selected {
                    ui.visuals().selection.bg_fill
                } else {
                    ui.visuals().widgets.hovered.bg_fill
                },
            );
        }
        if is_selected {
            ui.painter().line_segment(
                [
                    rect.left_top() + Vec2::new(1.5, 3.0),
                    rect.left_bottom() + Vec2::new(1.5, -3.0),
                ],
                Stroke::new(3.0, ACCENT),
            );
        }
        if response.has_focus() {
            ui.painter()
                .rect_stroke(rect, 4, Stroke::new(1.0, ACCENT), egui::StrokeKind::Inside);
        }
        let left = rect.left() + 8.0 + depth as f32 * 17.0;
        let expander =
            Rect::from_center_size(Pos2::new(left + 10.0, rect.center().y), Vec2::splat(24.0));
        let icon_rect =
            Rect::from_center_size(Pos2::new(left + 34.0, rect.center().y), Vec2::splat(20.0));
        let lock_rect = Rect::from_center_size(
            Pos2::new(rect.right() - 19.0, rect.center().y),
            Vec2::splat(28.0),
        );
        let direct_rect = lock_rect.translate(Vec2::new(-30.0, 0.0));
        #[cfg(test)]
        ui.ctx()
            .data_mut(|d| d.insert_temp(egui::Id::new(("test-row-lock", &item.id)), lock_rect));
        #[cfg(test)]
        ui.ctx()
            .data_mut(|d| d.insert_temp(egui::Id::new(("test-expander", &item.id)), expander));
        let mut over_control = false;
        if item.kind == "folder" {
            over_control =
                ui.input(|i| i.pointer.hover_pos().is_some_and(|p| expander.contains(p)));
            if crate::icons::at(
                ui,
                expander,
                if open { "down" } else { "chevron" },
                if open {
                    "Recolher pasta"
                } else {
                    "Expandir pasta"
                },
                false,
            )
            .clicked()
                && let Some(d) = self.doc_mut()
            {
                if open {
                    d.expanded.remove(&item.id);
                } else {
                    d.expanded.insert(item.id.clone());
                }
            }
            crate::icons::paint(ui, icon_rect, "folder", ui.visuals().weak_text_color());
        } else {
            ui.painter().rect_filled(icon_rect, 4, wheel::rgb(item.rgb));
            if self.picking.is_none() {
                over_control = ui.input(|i| {
                    i.pointer
                        .hover_pos()
                        .is_some_and(|p| lock_rect.contains(p) || direct_rect.contains(p))
                });
                if crate::icons::at(
                    ui,
                    lock_rect,
                    if item.locked || item.source_locked {
                        "lock"
                    } else {
                        "unlock"
                    },
                    if item.source_locked {
                        "Desvincular captura e desbloquear cor"
                    } else if item.locked {
                        "Desbloquear cor"
                    } else {
                        "Bloquear cor"
                    },
                    false,
                )
                .clicked()
                {
                    self.toggle_item_lock(p, item);
                }
                self.direct_lock_control(ui, item, direct_rect, ui.visuals().text_color());
            }
            ui.painter().text(
                Pos2::new(rect.right() - 78.0, rect.center().y),
                Align2::RIGHT_CENTER,
                color::hex(item.rgb),
                FontId::monospace(11.0),
                ui.visuals().weak_text_color(),
            );
            technical_badge(
                ui,
                p,
                item,
                Rect::from_center_size(
                    Pos2::new(rect.right() - 187.0, rect.center().y),
                    Vec2::splat(24.0),
                ),
                ui.visuals().text_color(),
            );
        }
        let name_width = (rect.width() - (left - rect.left()) - 253.0).max(35.0);
        let limit = (name_width / 6.7) as usize;
        let label = if item.label().chars().count() > limit {
            format!(
                "{}…",
                item.label()
                    .chars()
                    .take(limit.saturating_sub(1))
                    .collect::<String>()
            )
        } else {
            item.label()
        };
        ui.painter().text(
            Pos2::new(left + 51.0, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(13.0),
            ui.visuals().text_color(),
        );
        if !over_control {
            self.item_response(ui, p, item, response);
        }
        if recurse && open && item.kind == "folder" {
            for child in p
                .items
                .iter()
                .filter(|i| i.parent_id.as_deref() == Some(item.id.as_str()))
            {
                self.tree_item(ui, p, child, depth + 1, selected, true)
            }
        }
    }
    fn item_response(
        &mut self,
        ui: &mut egui::Ui,
        p: &Project,
        item: &Item,
        response: egui::Response,
    ) {
        if self
            .doc()
            .is_some_and(|d| d.reveal_item.as_deref() == Some(&item.id))
        {
            ui.scroll_to_rect(response.rect, Some(egui::Align::Center));
            if let Some(d) = self.doc_mut() {
                d.reveal_item = None;
            }
        }
        if self.picking.is_some() {
            if response.clicked() && item.is_color() {
                self.issue(Action::ChooseReplacement(item.id.clone()));
            }
            if response.double_clicked() && item.kind == "folder" {
                self.issue(Action::Folder(Some(item.id.clone())));
            }
            return;
        }
        if response.clicked() {
            let modifiers = ui.input(|i| i.modifiers);
            self.issue(Action::Select(
                item.id.clone(),
                modifiers.command,
                modifiers.shift,
            ));
        }
        if response.double_clicked() && item.kind == "folder" {
            self.issue(Action::Folder(Some(item.id.clone())))
        }
        if response.drag_started() {
            let mut ids = self.doc().map(|d| d.selected.clone()).unwrap_or_default();
            if !ids.contains(&item.id) {
                ids = vec![item.id.clone()];
                self.issue(Action::Select(item.id.clone(), false, false))
            }
            response.dnd_set_drag_payload(ids.clone());
            ui.ctx()
                .data_mut(|d| d.insert_temp(egui::Id::new("dragged-items"), ids));
        }
        if let Some(payload) = response.dnd_hover_payload::<Vec<String>>()
            && !payload.contains(&item.id)
        {
            ui.painter().rect_stroke(
                response.rect,
                8.0,
                Stroke::new(2.0, ACCENT),
                egui::StrokeKind::Inside,
            );
        }
        if let Some(payload) = response.dnd_release_payload::<Vec<String>>() {
            let ids = (*payload).clone();
            if !ids.contains(&item.id) {
                if item.kind == "folder" {
                    self.issue(Action::Core(Command::Move {
                        ids,
                        parent: Some(item.id.clone()),
                    }));
                } else {
                    let mut commands = vec![Command::Move {
                        ids: ids.clone(),
                        parent: item.parent_id.clone(),
                    }];
                    commands.extend(ids.into_iter().map(|id| Command::Reorder {
                        id,
                        before: item.id.clone(),
                    }));
                    self.issue(Action::Core(Command::Batch(commands)));
                }
            }
        }
        response.clone().on_hover_text(format!(
            "{}\n{}",
            item.label(),
            if item.is_color() {
                format!(
                    "{}\n{}\n{}",
                    color::hex(item.rgb),
                    item.description,
                    technical_summary(p, &item.id).unwrap_or_default()
                )
            } else {
                crate::i18n::tr("Clique duas vezes para abrir")
            }
        ));
        response.context_menu(|ui| {
            for (label, cut) in [("Copiar", false), ("Recortar", true)] {
                if ui.button(crate::i18n::tr(label)).clicked() {
                    if !self.doc().is_some_and(|d| d.selected.contains(&item.id)) {
                        self.issue(Action::Select(item.id.clone(), false, false));
                    }
                    self.issue(Action::Copy(cut));
                    ui.close();
                }
            }
            if ui.button(crate::i18n::tr("Colar")).clicked() {
                self.issue(Action::Paste);
                ui.close();
            }
            ui.separator();
            if item.kind == "folder" && ui.button(crate::i18n::tr("Abrir pasta")).clicked() {
                self.issue(Action::Folder(Some(item.id.clone())));
                ui.close();
            }
            if item.is_color()
                && ui
                    .button(crate::i18n::tr("Copiar HEX da prévia sRGB"))
                    .clicked()
            {
                ui.ctx().copy_text(color::hex(item.rgb));
                ui.close();
            }
            let ids = if self.doc().is_some_and(|d| d.selected.contains(&item.id)) {
                self.doc().map(|d| d.selected.clone()).unwrap_or_default()
            } else {
                vec![item.id.clone()]
            };
            if ui.button(crate::i18n::tr("Editar detalhes")).clicked() {
                self.issue(Action::Select(item.id.clone(), false, false));
                self.issue(Action::ShowPanel(Panel::Details));
                ui.close();
            }
            if ui.button(crate::i18n::tr("Duplicar")).clicked() {
                self.issue(Action::Core(Command::Duplicate(ids.clone())));
                ui.close();
            }
            if ui.button(crate::i18n::tr("Mover…")).clicked() {
                self.issue(Action::Show(Dialog::Move {
                    ids: ids.clone(),
                    parent: item.parent_id.clone(),
                }));
                ui.close();
            }
            if item.is_color() {
                if ui
                    .button(if item.source_locked {
                        "Desvincular e desbloquear"
                    } else if item.locked {
                        "Desbloquear"
                    } else {
                        "Bloquear"
                    })
                    .clicked()
                {
                    self.toggle_item_lock(p, item);
                    ui.close();
                }
                if let Some(summary) = technical_summary(p, &item.id) {
                    ui.label(RichText::new(summary).small());
                    if ui
                        .button(crate::i18n::tr("Ver relações técnicas…"))
                        .clicked()
                    {
                        self.issue(Action::Select(item.id.clone(), false, false));
                        self.issue(Action::ShowPanel(Panel::Links));
                        ui.close();
                    }
                    ui.separator();
                }
                if ui
                    .button(if item.direct_locked {
                        "Liberar edição direta"
                    } else {
                        "Bloquear apenas edição direta"
                    })
                    .clicked()
                {
                    self.issue(Action::Core(Command::DirectLock {
                        id: item.id.clone(),
                        locked: !item.direct_locked,
                    }));
                    ui.close();
                }
                if ui.button(crate::i18n::tr("Criar harmonia…")).clicked() {
                    self.issue(Action::Select(item.id.clone(), false, false));
                    let draft = colorcraft_core::engine::CompositionDraft {
                        originals: vec![item.id.clone()],
                        ..Default::default()
                    };
                    self.issue(Action::Show(Dialog::Composition(draft)));
                    ui.close();
                }
                if ui
                    .button(crate::i18n::tr("Criar vínculo / limite…"))
                    .clicked()
                {
                    if !self.doc().is_some_and(|d| d.selected.contains(&item.id)) {
                        self.issue(Action::Select(item.id.clone(), false, false));
                    }
                    self.open_link();
                    ui.close();
                }
            }
            ui.separator();
            if ui.button(crate::i18n::tr("Excluir…")).clicked() {
                self.issue(Action::Show(Dialog::Delete(ids)));
                ui.close();
            }
        });
        let _ = p;
    }
    fn color_panel(&mut self, ui: &mut egui::Ui) {
        let Some(p) = self.project().cloned() else {
            return;
        };
        let Some(item) = self
            .doc()
            .and_then(|d| d.active.as_deref())
            .and_then(|id| p.item(id))
            .cloned()
        else {
            ui.add_space(32.0);
            crate::theme::panel_title(ui, "Cor");
            ui.label(crate::i18n::tr("Selecione uma cor da paleta para editar."));
            return;
        };
        if !item.is_color() {
            ui.heading(item.label());
            ui.label(crate::i18n::tr(
                "Selecione uma cor; os detalhes da pasta estão no painel Detalhes.",
            ));
            return;
        }
        self.editable_title(ui, &item.label(), Some(item.id.clone()), 18.0);
        let native = item.native();
        let profile_label = native
            .profile_id
            .as_ref()
            .and_then(|id| p.profiles.iter().find(|pr| &pr.id == id))
            .map(|pr| pr.name.clone())
            .unwrap_or_else(|| match native.space.as_str() {
                "cmyk" => "CMYK sintético · 300% TAC".into(),
                "lab" => "Lab D50".into(),
                "display-p3" => "Display P3".into(),
                "gray" => "Gray Gamma 2.2".into(),
                _ => "sRGB IEC61966-2.1".into(),
            });
        ui.label(RichText::new(crate::i18n::tr(profile_label)).small().weak());
        let out = color::preview(&native, &p.profiles).is_ok_and(|rgb| !color::in_gamut(rgb));
        if out {
            ui.label(
                RichText::new(crate::i18n::tr(
                    "Fora do gamut sRGB · a prévia é uma aproximação",
                ))
                .color(crate::theme::accent_text(ui)),
            );
        }
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(Vec2::new(58.0, 42.0), Sense::hover());
            ui.painter().rect_filled(r, 6.0, wheel::rgb(item.rgb));
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(crate::i18n::tr(if item.is_srgb() {
                        "VALOR sRGB"
                    } else {
                        "PRÉVIA sRGB"
                    }))
                    .small()
                    .weak(),
                );
                if ui
                    .button(RichText::new(color::hex(item.rgb)).monospace().strong())
                    .on_hover_text(crate::i18n::tr("Copiar HEX da representação sRGB"))
                    .clicked()
                {
                    ui.ctx().copy_text(color::hex(item.rgb));
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if crate::icons::toggle(
                    ui,
                    if item.locked || item.source_locked {
                        "lock"
                    } else {
                        "unlock"
                    },
                    if item.source_locked {
                        "Desvincular captura e desbloquear cor"
                    } else if item.locked {
                        "Desbloquear cor"
                    } else {
                        "Bloquear cor"
                    },
                    item.locked || item.source_locked,
                )
                .clicked()
                {
                    self.toggle_item_lock(&p, &item);
                }
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::hover());
                self.direct_lock_control(ui, &item, rect, ui.visuals().text_color());
            });
        });
        if item.direct_locked {
            ui.label(
                RichText::new(crate::i18n::tr(
                    "Edição direta bloqueada · acompanha as cores relacionadas",
                ))
                .small()
                .color(ui.visuals().selection.stroke.color),
            );
        }
        ui.add_space(10.0);
        let mut related = vec![];
        let ids: Vec<_> = p
            .links
            .iter()
            .filter(|l| l.a == item.id || l.b == item.id)
            .map(|l| if l.a == item.id { &l.b } else { &l.a })
            .chain(
                p.harmonies
                    .iter()
                    .filter(|h| h.ids.contains(&item.id))
                    .flat_map(|h| h.ids.iter().filter(|id| *id != &item.id)),
            )
            .collect();
        for id in ids {
            if let Some(i) = p.item(id) {
                related.push(i.rgb)
            }
        }
        let enabled = !item.locked
            && !item.source_locked
            && !item.direct_locked
            && item.is_srgb()
            && !p
                .gradients
                .iter()
                .any(|g| g.ids.contains(&item.id) && !g.detached.contains(&item.id));
        let wheel = self.wheel.clone();
        if let Some(rgb) = wheel::show(ui, &wheel, item.rgb, &related, enabled) {
            self.issue(Action::Color {
                id: item.id.clone(),
                rgb,
                continuous: true,
            });
        }
        ui.add_space(6.0);
        if let Some((rgb, continuous)) = wheel::value_bar(ui, item.rgb, enabled) {
            self.issue(Action::Color {
                id: item.id.clone(),
                rgb,
                continuous,
            });
            if !continuous {
                self.issue(Action::Finish);
            }
        }
        ui.add_space(10.0);
        if !item.is_srgb() {
            ui.label(
                RichText::new(crate::i18n::tr(
                    "A roda usa sRGB. Os valores nativos abaixo permanecem preservados.",
                ))
                .small(),
            );
            if ui
                .button(crate::i18n::tr("Converter explicitamente para sRGB…"))
                .clicked()
            {
                self.issue(Action::Show(Dialog::Conversion {
                    id: item.id.clone(),
                    target: "srgb".into(),
                    assign: false,
                }));
            }
        }
        let hex_key = format!("{}:{}", item.id, color::hex(item.rgb));
        if self.hex_for != hex_key {
            self.hex_for = hex_key;
            self.hex_edit = color::hex(item.rgb);
        }
        ui.horizontal(|ui| {
            ui.label(crate::i18n::tr("HEX"));
            let response = ui.add_enabled(
                enabled,
                egui::TextEdit::singleline(&mut self.hex_edit)
                    .desired_width(130.0)
                    .font(egui::TextStyle::Monospace),
            );
            if (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                || ui
                    .add_enabled(enabled, egui::Button::new(crate::i18n::tr("Aplicar")))
                    .clicked()
            {
                match color::parse(&self.hex_edit) {
                    Ok(rgb) => {
                        self.issue(Action::Color {
                            id: item.id.clone(),
                            rgb,
                            continuous: false,
                        });
                        self.issue(Action::Finish);
                    }
                    Err(e) => self.error = Some(e),
                }
            }
        });
        if item.is_srgb() {
            ui.horizontal(|ui| {
                for model in ["HSV", "RGB", "HSL", "OKLCH"] {
                    ui.selectable_value(&mut self.component_mode, model.into(), model);
                }
            });
            let mut values = match self.component_mode.as_str() {
                "RGB" => item.rgb,
                "HSL" => {
                    let mut v = color::hsl(item.rgb);
                    v[1] *= 100.0;
                    v[2] *= 100.0;
                    v
                }
                "OKLCH" => {
                    let mut v = color::oklch(item.rgb);
                    v[0] *= 100.0;
                    v
                }
                _ => {
                    let mut v = color::hsv(item.rgb);
                    v[1] *= 100.0;
                    v[2] *= 100.0;
                    v
                }
            };
            let (labels, ranges) = match self.component_mode.as_str() {
                "RGB" => (["R", "G", "B"], [(0.0, 255.0); 3]),
                "HSL" => (
                    ["H", "S %", "L %"],
                    [(0.0, 360.0), (0.0, 100.0), (0.0, 100.0)],
                ),
                "OKLCH" => (["L %", "C", "H"], [(0.0, 100.0), (0.0, 0.4), (0.0, 360.0)]),
                _ => (
                    ["H", "S %", "V %"],
                    [(0.0, 360.0), (0.0, 100.0), (0.0, 100.0)],
                ),
            };
            let input_key = format!("{}:{}", item.id, self.component_mode);
            if self.component_edit_key != input_key {
                self.component_edit_key = input_key;
                self.component_inputs =
                    std::array::from_fn(|i| component_value(values[i], &self.component_mode));
            }
            let mut changed = false;
            let mut continuous = false;
            let mut done = false;
            ui.add_enabled_ui(enabled, |ui| {
                for c in 0..3 {
                    let hue_component = is_hue_component(&self.component_mode, c);
                    let row_width = ui.available_width().min(620.0);
                    let gap = ui.spacing().item_spacing.x;
                    let label_width = labels
                        .iter()
                        .map(|label| {
                            ui.painter()
                                .layout_no_wrap(
                                    (*label).into(),
                                    FontId::proportional(13.0),
                                    ui.visuals().text_color(),
                                )
                                .size()
                                .x
                        })
                        .fold(0.0_f32, f32::max)
                        + 6.0;
                    let available_for_columns = (row_width - gap * 2.0).max(0.0);
                    let label_width = label_width.min(available_for_columns * 0.28);
                    let value_width = 72.0_f32
                        .min((available_for_columns - label_width - 24.0).max(32.0));
                    let slider_width = (available_for_columns - label_width - value_width).max(0.0);
                    ui.allocate_ui_with_layout(
                        Vec2::new(ui.available_width(), 30.0),
                        egui::Layout::top_down(egui::Align::Center),
                        |outer| outer.allocate_ui_with_layout(
                        Vec2::new(row_width, 30.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |row| {
                            row.spacing_mut().slider_width = slider_width;
                        row.add_sized(Vec2::new(label_width, 28.0), egui::Label::new(labels[c]));
                        let slider = row.add_sized(
                            Vec2::new(slider_width, 28.0),
                            egui::Slider::new(&mut values[c], ranges[c].0..=ranges[c].1)
                                .clamping(egui::SliderClamping::Edits)
                                .show_value(false)
                                .max_decimals(if self.component_mode == "OKLCH" { 5 } else { 2 }),
                        );
                        changed |= slider.changed();
                        continuous |= slider.dragged();
                        done |= slider.drag_stopped();

                        let field = row.add_sized(
                            Vec2::new(value_width, 28.0),
                            egui::TextEdit::singleline(&mut self.component_inputs[c])
                                .horizontal_align(egui::Align::Center)
                                .vertical_align(egui::Align::Center)
                                .font(egui::TextStyle::Monospace),
                        ).on_hover_text(crate::i18n::tr("Digite um valor, + ou − para ajustar, * ou / para multiplicar ou dividir, e % para reduzir ou aumentar proporcionalmente. H aceita graus com ° ou º."));
                        if field.gained_focus()
                            && let Some(mut state) =
                                egui::text_edit::TextEditState::load(row.ctx(), field.id)
                        {
                            let end = egui::text::CCursor::new(
                                self.component_inputs[c].chars().count(),
                            );
                            state.cursor.set_char_range(Some(egui::text::CCursorRange::two(
                                egui::text::CCursor::new(0),
                                end,
                            )));
                            state.store(row.ctx(), field.id);
                        }
                        let enter = field.has_focus() && row.input(|i| i.key_pressed(egui::Key::Enter));
                        if field.lost_focus() || enter {
                            let current_input = values[c];
                            if let Some(input_value) = parse_component_input(&self.component_inputs[c], current_input, hue_component) {
                                let value = if hue_component {
                                    color::wrap(input_value)
                                } else {
                                    input_value.clamp(ranges[c].0, ranges[c].1)
                                };
                                if (value - values[c]).abs() > 1e-9 {
                                    values[c] = value;
                                    changed = true;
                                    done = true;
                                }
                            } else {
                                self.error = Some(crate::i18n::tr("Use um número, +/− para somar ou subtrair, * ou / para multiplicar ou dividir, e % para uma fração do valor atual. H também aceita °."));
                            }
                        }
                        if !field.has_focus() {
                            self.component_inputs[c] = component_value(values[c], &self.component_mode);
                        }
                    }));
                }
            });
            if changed {
                let rgb = match self.component_mode.as_str() {
                    "RGB" => values,
                    "HSL" => color::from_hsl([values[0], values[1] / 100.0, values[2] / 100.0]),
                    "OKLCH" => color::raw_oklch([values[0] / 100.0, values[1], values[2]]),
                    _ => color::from_hsv([values[0], values[1] / 100.0, values[2] / 100.0]),
                };
                if color::in_gamut(rgb) {
                    self.issue(Action::Color {
                        id: item.id.clone(),
                        rgb,
                        continuous,
                    })
                } else {
                    let mapped = color::map_chroma([values[0] / 100.0, values[1], values[2]]);
                    self.issue(Action::Show(Dialog::Gamut {
                        id: item.id.clone(),
                        original: item.rgb,
                        mapped,
                    }));
                }
            }
            if done {
                self.issue(Action::Finish)
            }
        } else {
            ui.add_space(8.0);
            ui.label(
                RichText::new(crate::i18n::tr("COMPONENTES NATIVOS"))
                    .small()
                    .strong(),
            );
            let mut value = native.clone();
            let mut changed = false;
            for (c, v) in value.components.iter_mut().enumerate() {
                let (label, range, speed) = if native.space == "lab" {
                    (
                        if c == 0 {
                            "L*"
                        } else if c == 1 {
                            "a*"
                        } else {
                            "b*"
                        },
                        if c == 0 { 0.0..=100.0 } else { -128.0..=127.0 },
                        0.1,
                    )
                } else {
                    let labels = if native.space == "cmyk" {
                        ["C", "M", "Y", "K"]
                    } else {
                        ["R", "G", "B", "Gray"]
                    };
                    (labels.get(c).copied().unwrap_or("Canal"), 0.0..=1.0, 0.001)
                };
                let row_width = ui.available_width().min(460.0);
                let gap = ui.spacing().item_spacing.x;
                let label_width = 72.0_f32.min(row_width * 0.4);
                let value_width = (row_width - gap - label_width).max(32.0);
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), 30.0),
                    egui::Layout::top_down(egui::Align::Center),
                    |outer| {
                        outer.allocate_ui_with_layout(
                            Vec2::new(row_width, 30.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |row| {
                                row.add_sized(
                                    Vec2::new(label_width, 28.0),
                                    egui::Label::new(label),
                                );
                                changed |= row
                                    .add_enabled_ui(
                                        !item.locked && !item.source_locked && !item.direct_locked,
                                        |value_ui| {
                                            value_ui.add_sized(
                                                Vec2::new(value_width, 28.0),
                                                egui::DragValue::new(v)
                                                    .speed(speed)
                                                    .range(range)
                                                    .clamp_existing_to_range(false)
                                                    .max_decimals(6),
                                            )
                                        },
                                    )
                                    .inner
                                    .changed();
                            },
                        )
                    },
                );
            }
            if changed {
                self.issue(Action::Preview(Command::SetNative {
                    id: item.id.clone(),
                    value,
                }));
            }
        }
        ui.add_space(10.0);
        if ui
            .button(crate::i18n::tr("Espaço de cor / perfil ICC…"))
            .clicked()
        {
            self.issue(Action::Show(Dialog::Conversion {
                id: item.id.clone(),
                target: if item.is_srgb() {
                    "display-p3".into()
                } else {
                    "srgb".into()
                },
                assign: false,
            }));
        }
        if ui.button(crate::i18n::tr("Explorar harmonias…")).clicked() {
            self.issue(Action::Show(Dialog::Composition(
                colorcraft_core::engine::CompositionDraft {
                    originals: vec![item.id.clone()],
                    ..Default::default()
                },
            )));
        }
        ui.separator();
        let hsl = color::hsl(item.rgb);
        let hsv = color::hsv(item.rgb);
        let oklch = color::oklch(item.rgb);
        for (label, value) in [
            (
                "RGB",
                format!(
                    "rgb({:.3} {:.3} {:.3})",
                    item.rgb[0], item.rgb[1], item.rgb[2]
                ),
            ),
            (
                "HSV",
                format!(
                    "hsv({:.2} {:.2}% {:.2}%)",
                    hsv[0],
                    hsv[1] * 100.0,
                    hsv[2] * 100.0
                ),
            ),
            (
                "HSL",
                format!(
                    "hsl({:.2} {:.2}% {:.2}%)",
                    hsl[0],
                    hsl[1] * 100.0,
                    hsl[2] * 100.0
                ),
            ),
            (
                "OKLCH",
                format!(
                    "oklch({:.4}% {:.6} {:.3})",
                    oklch[0] * 100.0,
                    oklch[1],
                    oklch[2]
                ),
            ),
        ] {
            ui.horizontal(|ui| {
                ui.label(RichText::new(label).small().weak());
                if ui
                    .small_button(RichText::new(&value).monospace().size(10.5))
                    .on_hover_text(crate::i18n::tr("Copiar representação da prévia sRGB"))
                    .clicked()
                {
                    ui.ctx().copy_text(value);
                }
            });
        }
    }
    fn links_panel(&mut self, ui: &mut egui::Ui) {
        let Some(p) = self.project().cloned() else {
            return;
        };
        let active = self.doc().and_then(|d| d.active.clone());
        crate::theme::panel_title(ui, "Relações da paleta");
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if crate::icons::button(ui, "link", "Criar vínculo, limite ou relação documental")
                .clicked()
            {
                self.open_link();
            }
            if crate::icons::button(ui, "spark", "Criar harmonia ou gradiente").clicked() {
                self.open_composition();
            }
        });
        ui.add_space(8.0);
        let focused = self.doc().and_then(|d| d.focused_relation.clone());
        let relevant = |ids: &[&str]| active.as_ref().is_none_or(|a| ids.contains(&a.as_str()));
        let mut any = false;
        for l in &p.links {
            if !relevant(&[&l.a, &l.b]) && focused.as_deref() != Some(&l.id) {
                continue;
            }
            any = true;
            let width = ui.available_width();
            crate::theme::card(ui).show(ui, |ui| {
                ui.set_width((width - 24.0).max(50.0));
                self.link_header(
                    ui,
                    &p,
                    &[l.a.clone(), l.b.clone()],
                    "VÍNCULO RGB / HSV",
                    &l.id,
                );
                ui.horizontal_wrapped(|ui| {
                    for c in &l.channels {
                        if let Some(r) = l.relations.get(&c.to_string()) {
                            ui.label(
                                RichText::new(format!(
                                    "{} {}{:.4}",
                                    ["R", "G", "B", "H", "S", "V"].get(*c).unwrap_or(&"?"),
                                    if r.mode == "ratio" { "×" } else { "Δ" },
                                    r.value
                                ))
                                .monospace()
                                .small(),
                            );
                        }
                    }
                });
                if ui
                    .small_button(crate::i18n::tr("Redefinir com os valores atuais"))
                    .clicked()
                {
                    self.issue(Action::Core(Command::RedefineLink(l.id.clone())));
                }
            });
            ui.add_space(6.0);
        }
        for h in &p.harmonies {
            if active.as_ref().is_some_and(|id| !h.ids.contains(id))
                && focused.as_deref() != Some(&h.id)
            {
                continue;
            }
            any = true;
            let width = ui.available_width();
            crate::theme::card(ui).show(ui, |ui| {
                ui.set_width((width - 24.0).max(50.0));
                self.link_header(ui, &p, &h.ids, color::harmony_name(&h.kind), &h.id);
                if let Some(parameter) =
                    colorcraft_core::catalog::harmony(&h.kind).and_then(|t| t.spread)
                {
                    if h.anchor_id.is_some() {
                        let mut spread = h.spread.unwrap_or(parameter.default);
                        let response = ui.add(
                            egui::Slider::new(&mut spread, parameter.min..=parameter.max)
                                .clamping(egui::SliderClamping::Edits)
                                .text(crate::i18n::tr(parameter.label))
                                .suffix(parameter.suffix),
                        );
                        if response.changed() {
                            self.issue(Action::Preview(Command::HarmonySpread {
                                id: h.id.clone(),
                                spread,
                            }));
                        }
                        let labels: Vec<_> = parameter
                            .suggestions
                            .iter()
                            .map(|n| format!("{n}{}", parameter.suffix))
                            .collect();
                        let choices: Vec<_> = parameter
                            .suggestions
                            .iter()
                            .zip(&labels)
                            .map(|(n, l)| (*n, l.as_str()))
                            .collect();
                        if crate::theme::chips(ui, &mut spread, &choices) {
                            self.issue(Action::Core(Command::HarmonySpread {
                                id: h.id.clone(),
                                spread,
                            }));
                        }
                        if response.drag_stopped() || response.lost_focus() {
                            self.issue(Action::Finish);
                        }
                    } else {
                        ui.label(
                            RichText::new(crate::i18n::tr(
                                "Harmonia antiga preservada sem inferir uma âncora.",
                            ))
                            .small()
                            .weak(),
                        );
                    }
                }
            });
            ui.add_space(6.0);
        }
        for g in &p.gradients {
            let mut members = vec![g.a.clone()];
            members.extend(g.ids.clone());
            members.push(g.b.clone());
            if active.as_ref().is_some_and(|id| !members.contains(id))
                && focused.as_deref() != Some(&g.id)
            {
                continue;
            }
            any = true;
            let width = ui.available_width();
            crate::theme::card(ui).show(ui, |ui| {
                ui.set_width((width - 24.0).max(50.0));
                self.link_header(ui, &p, &members, "GRADIENTE DINÂMICO", &g.id);
                ui.label(
                    RichText::new(crate::i18n::tr(format!(
                        "{} intermediárias · gamut: redução de cromaticidade",
                        g.ids.len()
                    )))
                    .small()
                    .weak(),
                );
                let mut space = g.space.clone();
                egui::ComboBox::from_id_salt(("grad-space", &g.id))
                    .selected_text(space.to_uppercase())
                    .show_ui(ui, |ui| {
                        for s in ["oklab", "oklch", "srgb"] {
                            ui.selectable_value(&mut space, s.into(), s.to_uppercase());
                        }
                    });
                if space != g.space {
                    self.issue(Action::Core(Command::GradientSpace {
                        id: g.id.clone(),
                        space,
                    }));
                }
                if let Some(id) = &active
                    && g.ids.contains(id)
                    && !g.detached.contains(id)
                    && ui
                        .small_button(crate::i18n::tr("Destacar esta cor do gradiente"))
                        .clicked()
                {
                    self.issue(Action::Core(Command::Detach {
                        gradient: g.id.clone(),
                        color: id.clone(),
                    }));
                }
            });
            ui.add_space(6.0);
        }
        for l in &p.limits {
            if !relevant(&[&l.a, &l.b]) && focused.as_deref() != Some(&l.id) {
                continue;
            }
            any = true;
            let width = ui.available_width();
            crate::theme::card(ui).show(ui, |ui| {
                ui.set_width((width - 24.0).max(50.0));
                self.link_header(
                    ui,
                    &p,
                    &[l.a.clone(), l.b.clone()],
                    "LIMITE DE CONTRASTE",
                    &l.id,
                );
                let ratio = p
                    .item(&l.a)
                    .zip(p.item(&l.b))
                    .map(|(a, b)| color::contrast(a.rgb, b.rgb))
                    .unwrap_or(1.0);
                ui.label(crate::i18n::tr(format!(
                    "Atual {:.3}:1 · mínimo {:.3}:1",
                    ratio, l.minimum
                )));
                ui.label(
                    RichText::new(crate::i18n::tr(
                        "WCAG 2.2 · cores opacas sRGB · limite duro",
                    ))
                    .small()
                    .weak(),
                );
            });
            ui.add_space(6.0);
        }
        for d in &p.documentary_relations {
            if !relevant(&[&d.from, &d.to]) {
                continue;
            }
            any = true;
            let width = ui.available_width();
            crate::theme::card(ui).show(ui, |ui| {
                ui.set_width((width - 24.0).max(50.0));
                ui.label(
                    RichText::new(crate::i18n::tr("DOCUMENTAL · NÃO RESTRINGE VALORES"))
                        .small()
                        .color(crate::theme::accent_text(ui)),
                );
                ui.label(format!(
                    "{} → {} · {}",
                    p.item(&d.from).map(Item::label).unwrap_or_default(),
                    p.item(&d.to).map(Item::label).unwrap_or_default(),
                    d.kind
                ));
                ui.label(&d.note);
                if ui
                    .small_button(crate::i18n::tr("Editar documentação…"))
                    .clicked()
                {
                    self.issue(Action::Show(Dialog::Documentation {
                        roles: p.roles.clone(),
                        modes: p.modes.clone(),
                        relations: p.documentary_relations.clone(),
                    }));
                }
            });
        }
        if !any {
            ui.add_space(26.0);
            ui.label(
                RichText::new(crate::i18n::tr("Cores que se movem juntas."))
                    .size(20.0)
                    .weak(),
            );
            ui.label(crate::i18n::tr(
                "Selecione cores e crie vínculos ou composições para começar.",
            ));
        }
    }
    fn link_header(
        &mut self,
        ui: &mut egui::Ui,
        p: &Project,
        ids: &[String],
        label: &str,
        relation: &str,
    ) {
        let focus = self
            .doc()
            .is_some_and(|d| d.focused_relation.as_deref() == Some(relation));
        if focus {
            let r = ui.available_rect_before_wrap();
            ui.painter().line_segment(
                [r.left_top(), r.left_top() + Vec2::new(r.width(), 0.0)],
                egui::Stroke::new(2.0, crate::theme::accent_text(ui)),
            );
            if self
                .doc()
                .is_some_and(|d| d.scroll_relation.as_deref() == Some(relation))
            {
                ui.scroll_to_rect(
                    egui::Rect::from_min_size(r.min, Vec2::new(r.width(), 30.0)),
                    Some(egui::Align::Center),
                );
                if let Some(d) = self.doc_mut() {
                    d.scroll_relation = None;
                }
            }
        }
        let header = ui.horizontal(|ui| {
            let icon = if label.contains("CONTRASTE") {
                "contrast"
            } else if label.contains("GRADIENTE") {
                "gradient"
            } else if label.contains("VÍNCULO") {
                "link"
            } else {
                "spark"
            };
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
            crate::icons::paint(ui, rect, icon, ui.visuals().text_color());
            ui.add(
                egui::Label::new(RichText::new(crate::i18n::tr(label)).size(12.0).strong())
                    .truncate(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if crate::icons::button(
                    ui,
                    "unlink",
                    "Remover somente esta relação, preservando as cores",
                )
                .clicked()
                {
                    self.issue(Action::Core(Command::RemoveRelation(relation.into())));
                }
            });
        });
        header.response.context_menu(|ui| {
            ui.label(crate::i18n::tr(label));
            if ui.button(crate::i18n::tr("Desfazer vínculo")).clicked() {
                self.issue(Action::Core(Command::RemoveRelation(relation.into())));
                ui.close();
            }
        });
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for id in ids {
                if let Some(item) = p.item(id) {
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::splat(26.0), Sense::click());
                    ui.painter().rect_filled(rect, 4.0, wheel::rgb(item.rgb));
                    if response.on_hover_text(item.label()).clicked() {
                        self.issue(Action::Select(id.clone(), false, false));
                    }
                }
            }
        });
        if ids.len() == 2 {
            ui.label(
                RichText::new(
                    ids.iter()
                        .filter_map(|id| p.item(id).map(Item::label))
                        .collect::<Vec<_>>()
                        .join(" ↔ "),
                )
                .small(),
            );
        }
    }
    fn details_panel(&mut self, ui: &mut egui::Ui) {
        let Some(p) = self.project().cloned() else {
            return;
        };
        let Some(item) = self
            .doc()
            .and_then(|d| d.active.as_deref().or(d.folder.as_deref()))
            .and_then(|id| p.item(id))
            .cloned()
        else {
            crate::theme::panel_title(ui, "Detalhes do projeto");
            ui.label(&p.name);
            ui.label(&p.description);
            if ui
                .button(crate::i18n::tr("Editar nome e descrição…"))
                .clicked()
            {
                self.issue(Action::Show(Dialog::Project {
                    name: p.name.clone(),
                    description: p.description.clone(),
                }));
            }
            ui.add_space(14.0);
            self.modes_editor(ui, &p);
            ui.add_space(10.0);
            if crate::icons::text_button(ui, "file", "Relações documentais…").clicked() {
                self.issue(Action::Show(Dialog::Documentation {
                    roles: p.roles.clone(),
                    modes: p.modes.clone(),
                    relations: p.documentary_relations.clone(),
                }));
            }
            return;
        };
        let mut draft = item.clone();
        let mut changed = false;
        let mut finish = false;
        crate::theme::panel_title(
            ui,
            if item.kind == "folder" {
                "Detalhes da pasta"
            } else {
                "Detalhes da cor"
            },
        );
        ui.label(
            RichText::new(crate::i18n::tr("ORGANIZAÇÃO E SIGNIFICADO"))
                .small()
                .color(crate::theme::accent_text(ui)),
        );
        ui.add_space(10.0);
        for (label, text) in [
            ("Nome", &mut draft.name),
            ("Nome CSS", &mut draft.css_name),
            ("Uso", &mut draft.usage),
        ] {
            ui.label(crate::i18n::tr(label));
            let r = ui.add(egui::TextEdit::singleline(text).desired_width(f32::INFINITY));
            changed |= r.changed();
            finish |= r.lost_focus();
        }
        ui.label(crate::i18n::tr("Descrição"));
        let r = ui.add(
            egui::TextEdit::multiline(&mut draft.description)
                .desired_rows(3)
                .desired_width(f32::INFINITY),
        );
        changed |= r.changed();
        finish |= r.lost_focus();
        ui.label(crate::i18n::tr("Tags"));
        let available: Vec<String> = p
            .items
            .iter()
            .flat_map(|i| i.tags.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        let tag_changed = crate::widgets::tag_field(
            ui,
            egui::Id::new(("item-tags", &p.id, &item.id)),
            &mut draft.tags,
            &available,
        );
        changed |= tag_changed;
        finish |= tag_changed;
        ui.add_space(9.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(crate::i18n::tr("CAMPOS PERSONALIZADOS"))
                    .small()
                    .strong(),
            );
            if ui.small_button(crate::i18n::tr("+")).clicked() {
                let mut key = "Campo".to_string();
                let mut n = 1;
                while draft.custom.contains_key(&key) {
                    n += 1;
                    key = format!("Campo {n}");
                }
                draft.custom.insert(key, serde_json::json!(""));
                changed = true;
            }
        });
        let fields: Vec<_> = draft.custom.clone().into_iter().collect();
        for (key, value) in fields {
            ui.horizontal(|ui| {
                ui.label(&key);
                let mut text = value
                    .as_str()
                    .map(String::from)
                    .unwrap_or_else(|| value.to_string());
                let r = ui.add(
                    egui::TextEdit::singleline(&mut text)
                        .desired_width((ui.available_width() - 33.0).max(80.0)),
                );
                if r.changed() {
                    draft.custom.insert(key.clone(), serde_json::json!(text));
                    changed = true
                }
                finish |= r.lost_focus();
                if ui.small_button(crate::i18n::tr("×")).clicked() {
                    draft.custom.remove(&key);
                    changed = true;
                }
            });
        }
        if changed {
            self.issue(Action::Preview(Command::Metadata(draft)));
        }
        if finish {
            self.issue(Action::Finish)
        }
        ui.separator();
        ui.label(
            RichText::new(crate::i18n::tr("PAPÉIS ASSOCIADOS"))
                .small()
                .strong(),
        );
        for role in p.roles.iter().filter(|r| r.color_id == item.id) {
            ui.label(crate::i18n::tr(format!("{} · padrão", role.name)));
        }
        for mode in &p.modes {
            for (role, id) in &mode.overrides {
                if id == &item.id
                    && let Some(r) = p.roles.iter().find(|r| &r.id == role)
                {
                    ui.label(crate::i18n::tr(format!("{} · {}", r.name, mode.name)));
                }
            }
        }
        if crate::icons::text_button(ui, "roles", "Abrir papéis semânticos").clicked() {
            self.issue(Action::ShowPanel(Panel::Roles));
        }
        ui.add_space(18.0);
        ui.label(
            RichText::new(crate::i18n::tr(format!("ID: {}", item.id)))
                .monospace()
                .size(10.0)
                .weak(),
        );
    }
}

#[cfg(test)]
mod component_input_tests {
    use super::parse_component_input;

    #[test]
    fn relative_component_edits_use_the_previous_value() {
        assert_eq!(parse_component_input("20%", 80.0, false), Some(64.0));
        assert_eq!(parse_component_input("-25", 80.0, false), Some(55.0));
        assert_eq!(parse_component_input("+30", 80.0, false), Some(110.0));
        assert_eq!(parse_component_input("*2", 80.0, false), Some(160.0));
        assert_eq!(parse_component_input("/2", 80.0, false), Some(40.0));
        assert_eq!(parse_component_input("+20%", 80.0, false), Some(96.0));
    }

    #[test]
    fn hue_inputs_use_degrees_directly_and_wrap_at_three_hundred_sixty() {
        let thirty_degrees = parse_component_input("30º", 180.0, true).unwrap_or(f64::NAN);
        assert!((thirty_degrees - 30.0).abs() < 1e-10);
        let fifteen_more_degrees = parse_component_input("+15°", 180.0, true).unwrap_or(f64::NAN);
        assert!((fifteen_more_degrees - 195.0).abs() < 1e-10);
        assert_eq!(parse_component_input("20%", 180.0, true), Some(144.0));
        assert_eq!(parse_component_input("+15", 180.0, true), Some(195.0));
        assert_eq!(parse_component_input("370", 180.0, true), Some(10.0));
        assert!(parse_component_input("30°", 50.0, false).is_none());
        assert!(parse_component_input("/0", 50.0, false).is_none());
    }
}
