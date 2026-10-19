//! Capture workbench. Panels reserve the footer before the canvas is laid out.
use crate::{
    app::{ACCENT, SourceWorkbench, WorkbenchTool},
    icons,
};
use colorcraft_core::{CaptureMethod, capture, color};
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke, Vec2};

#[derive(Default)]
pub struct Outcome {
    pub apply: bool,
    pub cancel: bool,
}

pub fn show(ui: &mut egui::Ui, wb: &mut SourceWorkbench) -> Outcome {
    if wb.pending_delete.is_some()
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
    {
        wb.pending_delete = None;
    }
    let result = ui
        .add_enabled_ui(wb.pending_delete.is_none(), |ui| workbench(ui, wb))
        .inner;
    confirm_delete(ui.ctx(), wb);
    result
}

fn confirm_delete(ctx: &egui::Context, wb: &mut SourceWorkbench) {
    let Some(index) = wb.pending_delete else {
        return;
    };
    let Some(cap) = wb.source.captures.get(index) else {
        wb.pending_delete = None;
        return;
    };
    let name = cap.name.clone();
    let linked = cap.color_id.is_some();
    egui::Modal::new(egui::Id::new("capture-delete-confirmation"))
        .frame(egui::Frame::window(&ctx.global_style()).inner_margin(20))
        .show(ctx, |ui| {
            ui.set_width((ctx.content_rect().width() - 64.0).clamp(200.0, 380.0));
            crate::theme::section(ui, "trash", "Remover captura");
            ui.add_space(12.0);
            ui.label(RichText::new(&name).strong());
            ui.label(crate::i18n::tr(if linked {
                "Remover a captura preservando a cor na paleta?"
            } else {
                "Remover esta captura do rascunho?"
            }));
            ui.add_space(16.0);
            ui.horizontal_wrapped(|ui| {
                let keep = ui.button(crate::i18n::tr("Manter captura"));
                record(ui, "delete-keep", keep.rect);
                if keep.clicked() {
                    wb.pending_delete = None;
                }
                let remove = ui.add(crate::theme::primary("Remover captura"));
                record(ui, "delete-remove", remove.rect);
                if remove.clicked() {
                    wb.source.captures.remove(index);
                    wb.selected = wb.selected.min(wb.source.captures.len().saturating_sub(1));
                    wb.pending_delete = None;
                    wb.moving_capture = None;
                    wb.eraser_last = None;
                    wb.dirty = true;
                }
            });
        });
}

fn workbench(ui: &mut egui::Ui, wb: &mut SourceWorkbench) -> Outcome {
    let mut outcome = Outcome::default();
    egui::Panel::bottom("capture-footer")
        .exact_size(44.0)
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(crate::i18n::tr(if wb.dirty {
                    "Alterações pendentes"
                } else {
                    "Rascunho de captura"
                }));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let ready = wb.color_ready()
                        && !wb.reload_page
                        && wb.pending_delete.is_none()
                        && wb.source.captures.iter().all(|c| {
                            !matches!(c.method, CaptureMethod::Brush { .. })
                                || !c.strokes.is_empty()
                        });
                    let apply = ui.add_enabled(
                        ready,
                        egui::Button::new(crate::i18n::tr("Aplicar capturas")),
                    );
                    record(ui, "apply", apply.rect);
                    if apply.clicked() {
                        outcome.apply = true;
                    }
                    let cancel = ui.button(crate::i18n::tr("Cancelar"));
                    record(ui, "cancel", cancel.rect);
                    if cancel.clicked() {
                        outcome.cancel = true;
                    }
                });
            });
        });
    egui::Panel::top("capture-toolbar").show(ui, |ui| {
        ui.add(egui::Label::new(RichText::new(&wb.source.name).strong()).truncate())
            .on_hover_text(&wb.source.path);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(crate::i18n::tr("Capturar cores da fonte"))
                    .small()
                    .weak(),
            );
            if wb.source.page_count > 1 {
                ui.separator();
                ui.label(crate::i18n::tr("Página"));
                let mut page = wb.page + 1;
                if ui
                    .add(egui::DragValue::new(&mut page).range(1..=wb.source.page_count))
                    .changed()
                {
                    wb.page = page - 1;
                    wb.reload_page = true;
                    wb.moving_capture = None;
                }
                ui.label(crate::i18n::tr(format!("de {}", wb.source.page_count)));
            }
        });
    });
    if let Some(warning) = wb.color_warning.clone() {
        egui::Panel::top("capture-color-notice").show(ui, |ui| {
            if wb.color_warning_accepted {
                ui.label(
                    RichText::new(crate::i18n::tr(
                        "Perfil ICC · captura dos pixels da prévia autorizada",
                    ))
                    .small()
                    .color(ACCENT),
                )
                .on_hover_text(&warning);
            } else {
                egui::ScrollArea::vertical()
                    .id_salt("color-notice-scroll")
                    .max_height(130.0)
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(crate::i18n::tr("Escolha como continuar"))
                                .strong()
                                .color(ACCENT),
                        );
                        ui.label(&warning);
                        ui.horizontal_wrapped(|ui| {
                            if ui
                                .button(crate::i18n::tr("Usar cores da prévia PDFium"))
                                .clicked()
                            {
                                wb.color_warning_accepted = true;
                            }
                            if ui
                                .button(crate::i18n::tr("Cancelar para converter uma cópia"))
                                .clicked()
                            {
                                outcome.cancel = true;
                            }
                        });
                    });
            }
        });
    }
    let ready = wb.color_ready() && !wb.reload_page;
    egui::Panel::left("capture-tools").exact_size(44.0).show(ui, |ui| {
        ui.vertical_centered(|ui| {
            for (tool, icon, label) in [
                (WorkbenchTool::Dropper, "dropper", "Conta-gotas · selecione ou arraste marcadores; clique fora para criar"),
                (WorkbenchTool::Brush, "brush", "Pincel de máscara · pinte uma nova captura ou continue a máscara selecionada"),
                (WorkbenchTool::Eraser, "eraser", "Borracha por traço · apaga traços completos da máscara selecionada"),
                (WorkbenchTool::Hand, "hand", "Mão · arraste para deslocar a imagem"),
                (WorkbenchTool::Zoom, "zoom", "Lupa · clique para ampliar; Alt ou botão direito para reduzir"),
            ] {
                let response = icons::toggle(ui, icon, label, wb.tool == tool);
                record(ui, icon, response.rect);
                if response.clicked() { wb.end_brush(); wb.tool = tool; wb.moving_capture = None; wb.eraser_last = None; }
            }
            ui.separator();
            if icons::button(ui, "fit", "Ajustar à janela e centralizar").clicked() { wb.zoom = 1.0; wb.pan = Vec2::ZERO; }
            if icons::button(ui, "plus", "Ampliar imagem").clicked() { wb.zoom = (wb.zoom * 1.25).min(16.0); }
            if icons::button(ui, "minus", "Reduzir imagem").clicked() { wb.zoom = (wb.zoom / 1.25).max(0.1); }
        });
    });
    let width = (ui.available_width() * 0.34).clamp(230.0, 340.0);
    egui::Panel::right("capture-inspector")
        .exact_size(width)
        .show(ui, |ui| {
            ui.add_enabled_ui(ready, |ui| inspector(ui, wb));
        });
    egui::CentralPanel::default().show(ui, |ui| {
        ui.add_enabled_ui(ready, |ui| canvas(ui, wb));
    });
    outcome
}

fn inspector(ui: &mut egui::Ui, wb: &mut SourceWorkbench) {
    let height = (ui.available_height() * 0.42).clamp(100.0, 300.0);
    egui::Panel::top("capture-list-panel")
        .resizable(true)
        .default_size(height)
        .size_range(90.0..=(ui.available_height() * 0.65).max(90.0))
        .show(ui, |ui| {
            capture_list(ui, wb);
        });
    egui::CentralPanel::default().show(ui, |ui| {
        ui.heading(crate::i18n::tr("Opções"));
        capture_options(ui, wb);
    });
}

fn capture_list(ui: &mut egui::Ui, wb: &mut SourceWorkbench) {
    ui.heading(crate::i18n::tr(format!(
        "Capturas · {}",
        wb.source.captures.len()
    )));
    ui.horizontal(|ui| {
        let label = if wb.tool == WorkbenchTool::Brush {
            "Nova máscara"
        } else {
            "Nova amostra"
        };
        let new = ui.button(format!("     {}", crate::i18n::tr(label)));
        icons::paint(
            ui,
            Rect::from_center_size(
                new.rect.left_center() + Vec2::new(14.0, 0.0),
                Vec2::splat(16.0),
            ),
            "plus",
            ui.visuals().text_color(),
        );
        record(ui, "new-capture", new.rect);
        if new
            .on_hover_text(crate::i18n::tr(
                "Criar captura independente, preservando a selecionada",
            ))
            .clicked()
        {
            wb.new_capture([0.5, 0.5]);
            if wb.tool == WorkbenchTool::Brush {
                wb.source.captures[wb.selected].method = CaptureMethod::Brush {
                    statistic: "mean".into(),
                };
            }
        }
        if icons::enabled_button(
            ui,
            !wb.source.captures.is_empty(),
            "trash",
            "Remover captura selecionada",
        )
        .clicked()
        {
            wb.pending_delete = Some(wb.selected);
        }
    });
    // The list and the fields scroll independently; a long list never hides the inspector.
    egui::ScrollArea::vertical()
        .id_salt("capture-list")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for index in 0..wb.source.captures.len() {
                let cap = &wb.source.captures[index];
                let (row, response) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 34.0), Sense::click());
                if index == wb.selected || response.hovered() {
                    ui.painter().rect_filled(
                        row,
                        4.0,
                        if index == wb.selected {
                            ui.visuals().selection.bg_fill
                        } else {
                            ui.visuals().widgets.hovered.bg_fill
                        },
                    );
                }
                let rgb = cap.value.map(|v| v.round().clamp(0.0, 255.0) as u8);
                ui.painter().circle_filled(
                    row.left_center() + Vec2::new(12.0, 0.0),
                    7.0,
                    Color32::from_rgb(rgb[0], rgb[1], rgb[2]),
                );
                icons::paint(
                    ui,
                    Rect::from_center_size(
                        row.left_center() + Vec2::new(34.0, 0.0),
                        Vec2::splat(16.0),
                    ),
                    if matches!(cap.method, CaptureMethod::Brush { .. }) {
                        "brush"
                    } else {
                        "dropper"
                    },
                    ui.visuals().text_color(),
                );
                let text = Rect::from_min_max(
                    row.min + Vec2::new(48.0, 0.0),
                    row.max - Vec2::new(4.0, 0.0),
                );
                // Paint the label without registering another interactive widget above the row.
                let galley = ui.painter().layout(
                    cap.name.clone(),
                    egui::FontId::proportional(13.0),
                    ui.visuals().text_color(),
                    text.width(),
                );
                ui.painter().with_clip_rect(text).galley(
                    text.left_center() - Vec2::new(0.0, galley.size().y / 2.0),
                    galley,
                    ui.visuals().text_color(),
                );
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::WidgetType::SelectableLabel,
                        ui.is_enabled(),
                        &cap.name,
                    )
                });
                let response = response.on_hover_text(crate::i18n::tr(format!(
                    "{} · {} · página {}",
                    cap.name,
                    color::hex(cap.value),
                    cap.page + 1
                )));
                if response.clicked() {
                    wb.select_capture(index);
                }
            }
        });
}

fn capture_options(ui: &mut egui::Ui, wb: &mut SourceWorkbench) {
    egui::ScrollArea::vertical().id_salt("capture-properties").auto_shrink([false, false]).show(ui, |ui| {
        if wb.tool == WorkbenchTool::Eraser {
            ui.label(RichText::new(crate::i18n::tr("Borracha por traço")).strong());
            ui.label(crate::i18n::tr("Passe sobre um traço para apagá-lo inteiro. Somente a máscara selecionada é afetada."));
            ui.add_space(12.0);
        }
        if wb.tool == WorkbenchTool::Brush {
            ui.label(RichText::new(crate::i18n::tr("Pincel de máscara")).strong());
            ui.add(egui::Slider::new(&mut wb.brush_radius, 1.0..=128.0).text(crate::i18n::tr("Raio (px)")));
            ui.label(RichText::new(crate::i18n::tr("Uma captura pontual é preservada ao começar uma máscara. Selecione uma máscara para continuar pintando nela.")).small().weak());
        }
        let Some(cap) = wb.source.captures.get_mut(wb.selected) else { ui.label(crate::i18n::tr("Clique na imagem ou pinte uma máscara para começar.")); return; };
        ui.label(crate::i18n::tr("Nome"));
        if ui.add(egui::TextEdit::singleline(&mut cap.name).desired_width(f32::INFINITY)).changed() { wb.dirty = true; }
        ui.label(crate::i18n::tr("Método"));
        let before = cap.method.clone();
        let mut mode = match cap.method { CaptureMethod::Pixel => 0, CaptureMethod::Area { .. } => 1, CaptureMethod::Brush { .. } => 2 };
        egui::ComboBox::from_id_salt("capture-method").selected_text(crate::i18n::tr(["Pixel exato", "Área próxima", "Máscara de pincel"][mode])).show_ui(ui, |ui| {
            ui.selectable_value(&mut mode, 0, crate::i18n::tr("Pixel exato")); ui.selectable_value(&mut mode, 1, crate::i18n::tr("Área próxima")); ui.selectable_value(&mut mode, 2, crate::i18n::tr("Máscara de pincel"));
        });
        if mode == 0 { cap.method = CaptureMethod::Pixel; }
        if mode == 1 && !matches!(cap.method, CaptureMethod::Area { .. }) { cap.method = CaptureMethod::Area { shape: "circle".into(), statistic: "mean".into(), radius: 5 }; }
        if mode == 2 && !matches!(cap.method, CaptureMethod::Brush { .. }) { cap.method = CaptureMethod::Brush { statistic: "mean".into() }; }
        match &mut cap.method {
            CaptureMethod::Area { shape, statistic, radius } => {
                egui::ComboBox::from_id_salt("capture-shape").selected_text(crate::i18n::tr(if shape == "circle" { "Círculo" } else { "Quadrado" })).show_ui(ui, |ui| { ui.selectable_value(shape, "circle".into(), crate::i18n::tr("Círculo")); ui.selectable_value(shape, "square".into(), crate::i18n::tr("Quadrado")); });
                statistic_combo(ui, statistic);
                ui.add(egui::Slider::new(radius, 1..=128).text(crate::i18n::tr("Raio (px)")));
            }
            CaptureMethod::Brush { statistic } => {
                statistic_combo(ui, statistic);
                if ui.add_enabled(!cap.strokes.is_empty(), egui::Button::new(crate::i18n::tr("Limpar máscara"))).clicked() { cap.strokes.clear(); wb.dirty = true; }
                if cap.strokes.is_empty() { ui.colored_label(ACCENT, crate::i18n::tr("Pinte a máscara antes de aplicar.")); }
            }
            CaptureMethod::Pixel => { ui.label(RichText::new(crate::i18n::tr("Pixel indicado pelo marcador.")).small().weak()); }
        }
        if before != cap.method { wb.dirty = true; }
        if before != cap.method && cap.page == wb.page && !wb.reload_page {
            match capture::sample(wb.image.width(), wb.image.height(), wb.image.as_raw(), cap) {
                Ok(value) => { if cap.value != value { wb.dirty = true; cap.value = value; } }
                Err(error) => { ui.colored_label(ACCENT, error); }
            }
        }
        ui.separator();
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(48.0, 32.0), Sense::hover());
            ui.painter().rect_filled(rect, 4.0, Color32::from_rgb(cap.value[0] as u8, cap.value[1] as u8, cap.value[2] as u8));
            ui.monospace(color::hex(cap.value));
        });
        ui.label(RichText::new(crate::i18n::tr(if cap.color_id.is_some() { "Cor vinculada · será atualizada ao aplicar." } else { "Nova cor na raiz ao aplicar." })).small().weak());
    });
}

fn statistic_combo(ui: &mut egui::Ui, value: &mut String) {
    egui::ComboBox::from_id_salt("capture-statistic")
        .selected_text(crate::i18n::tr(if value == "mean" {
            "Média linear sRGB"
        } else {
            "Mediana OKLab"
        }))
        .show_ui(ui, |ui| {
            ui.selectable_value(value, "mean".into(), crate::i18n::tr("Média linear sRGB"));
            ui.selectable_value(value, "median".into(), crate::i18n::tr("Mediana OKLab"));
        });
}

fn canvas(ui: &mut egui::Ui, wb: &mut SourceWorkbench) {
    egui::Panel::bottom("capture-canvas-help").show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.monospace(format!("{}%", (wb.zoom * 100.0).round() as u32));
            ui.label(RichText::new(crate::i18n::tr(match wb.tool {
                WorkbenchTool::Dropper => "Clique no marcador para selecionar; arraste para mover. Clique fora para criar.",
                WorkbenchTool::Brush => "Arraste para pintar uma máscara. Capturas pontuais são preservadas.",
                WorkbenchTool::Eraser => "Passe sobre um traço para apagá-lo inteiro. Somente a máscara selecionada é afetada.",
                WorkbenchTool::Hand => "Arraste a imagem para reposicionar. Ajustar centraliza novamente.",
                WorkbenchTool::Zoom => "Clique para ampliar. Alt + clique ou botão direito reduz. A roda do mouse também ajusta o zoom.",
            })).small().weak());
        });
    });
    let available = ui.available_size().max(Vec2::splat(1.0));
    let (viewport, response) = ui.allocate_exact_size(available, Sense::click_and_drag());
    let painter = ui
        .painter()
        .with_clip_rect(viewport.intersect(ui.clip_rect()));
    painter.rect_filled(
        viewport,
        0.0,
        Color32::from_gray(if ui.visuals().dark_mode { 24 } else { 220 }),
    );
    let fit = ((available.x - 16.0).max(1.0) / wb.image.width() as f32)
        .min((available.y - 16.0).max(1.0) / wb.image.height() as f32);
    let pointer = ui.input(|i| i.pointer.interact_pos());
    if ui.is_enabled() {
        if wb.tool == WorkbenchTool::Hand && response.dragged() {
            wb.pan += ui.input(|i| i.pointer.delta());
        }
        let scroll = if response.hovered() {
            ui.input(|i| i.smooth_scroll_delta.y)
        } else {
            0.0
        };
        let zoom_click =
            wb.tool == WorkbenchTool::Zoom && (response.clicked() || response.secondary_clicked());
        if scroll.abs() > 0.0 || zoom_click {
            let old = wb.zoom;
            let factor = if scroll.abs() > 0.0 {
                (scroll * 0.003).exp()
            } else if response.secondary_clicked() || ui.input(|i| i.modifiers.alt) {
                0.8
            } else {
                1.25
            };
            wb.zoom = (old * factor).clamp(0.1, 16.0);
            let anchor = pointer.unwrap_or(viewport.center()) - viewport.center();
            wb.pan = anchor - (anchor - wb.pan) * (wb.zoom / old);
        }
    }
    let draw = Vec2::new(wb.image.width() as f32, wb.image.height() as f32) * fit * wb.zoom;
    let rect = Rect::from_center_size(viewport.center() + wb.pan, draw);
    record(ui, "canvas", rect);
    record(ui, "image-panel", viewport);
    painter.image(
        wb.texture.id(),
        rect,
        Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
        Color32::WHITE,
    );
    let map = |pos: Pos2| {
        [
            ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0),
            ((pos.y - rect.top()) / rect.height()).clamp(0.0, 1.0),
        ]
    };
    let point = |p: [f32; 2]| rect.min + Vec2::new(p[0] * rect.width(), p[1] * rect.height());
    if ui.is_enabled() {
        if response.contains_pointer()
            && ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary))
            && wb.tool == WorkbenchTool::Dropper
            && let Some(pos) = pointer
        {
            wb.moving_capture = wb.marker_at(rect, pos);
            if let Some(index) = wb.moving_capture {
                wb.select_capture(index);
            }
        }
        match wb.tool {
            WorkbenchTool::Dropper => {
                if response.drag_started()
                    && wb.moving_capture.is_none()
                    && let Some(origin) = ui.input(|i| i.pointer.press_origin())
                {
                    wb.moving_capture = wb.marker_at(rect, origin);
                }
                if response.dragged()
                    && let (Some(index), Some(pos)) = (wb.moving_capture, pointer)
                {
                    wb.move_capture(index, map(pos));
                }
                if response.clicked()
                    && let Some(pos) = pointer
                    && rect.contains(pos)
                {
                    if let Some(index) = wb.moving_capture.or_else(|| wb.marker_at(rect, pos)) {
                        wb.select_capture(index);
                    } else {
                        wb.new_capture(map(pos));
                    }
                }
            }
            WorkbenchTool::Brush => {
                if response.drag_started()
                    && let Some(origin) = ui.input(|i| i.pointer.press_origin())
                    && rect.contains(origin)
                {
                    wb.start_brush(map(origin));
                }
                if response.dragged()
                    && let Some(pos) = pointer
                    && rect.contains(pos)
                {
                    wb.continue_brush(map(pos));
                }
                if response.clicked()
                    && let Some(pos) = pointer
                    && rect.contains(pos)
                {
                    wb.start_brush(map(pos));
                    wb.end_brush();
                }
            }
            WorkbenchTool::Eraser => {
                if response.is_pointer_button_down_on()
                    && ui.input(|i| i.pointer.primary_down())
                    && let Some(pos) = pointer
                    && rect.contains(pos)
                {
                    let current = map(pos);
                    let previous = wb.eraser_last.unwrap_or(current);
                    crate::capture_mask::erase(
                        wb,
                        previous,
                        current,
                        5.0 * wb.image.width() as f32 / rect.width(),
                    );
                    wb.eraser_last = Some(current);
                }
            }
            WorkbenchTool::Hand | WorkbenchTool::Zoom => {}
        }
    }
    crate::capture_mask::paint(ui, wb, &painter, rect);
    for (index, cap) in wb
        .source
        .captures
        .iter()
        .enumerate()
        .filter(|(_, c)| c.page == wb.page)
    {
        let selected = index == wb.selected;
        let pos = point(cap.position);
        painter.circle_filled(
            pos,
            7.0,
            Color32::from_rgb(cap.value[0] as u8, cap.value[1] as u8, cap.value[2] as u8),
        );
        painter.circle_stroke(pos, 9.0, Stroke::new(2.0, Color32::BLACK));
        painter.circle_stroke(
            pos,
            11.0,
            Stroke::new(
                if selected { 2.5 } else { 1.5 },
                if selected { ACCENT } else { Color32::WHITE },
            ),
        );
        painter.text(
            pos + Vec2::new(13.0, -13.0),
            egui::Align2::LEFT_BOTTOM,
            (index + 1).to_string(),
            egui::FontId::monospace(12.0),
            Color32::WHITE,
        );
    }
    if let Some(pos) = pointer {
        if response.hovered() {
            ui.ctx().set_cursor_icon(match wb.tool {
                WorkbenchTool::Hand => {
                    if response.dragged() {
                        egui::CursorIcon::Grabbing
                    } else {
                        egui::CursorIcon::Grab
                    }
                }
                WorkbenchTool::Zoom => {
                    if ui.input(|i| i.modifiers.alt) {
                        egui::CursorIcon::ZoomOut
                    } else {
                        egui::CursorIcon::ZoomIn
                    }
                }
                _ => egui::CursorIcon::Crosshair,
            });
        }
        if wb.tool == WorkbenchTool::Brush && response.hovered() && rect.contains(pos) {
            let radius = (wb.brush_radius * rect.width() / wb.image.width() as f32).max(2.0);
            painter.circle_stroke(pos, radius + 1.0, Stroke::new(2.0, Color32::BLACK));
            painter.circle_stroke(pos, radius, Stroke::new(1.5, Color32::WHITE));
        }
        if wb.tool == WorkbenchTool::Eraser && response.hovered() && rect.contains(pos) {
            painter.circle_stroke(pos, 7.0, Stroke::new(3.0, Color32::BLACK));
            painter.circle_stroke(pos, 7.0, Stroke::new(1.5, Color32::WHITE));
            painter.line_segment(
                [pos - Vec2::new(3.0, 0.0), pos + Vec2::new(3.0, 0.0)],
                Stroke::new(1.5, Color32::WHITE),
            );
        }
        if wb.moving_capture.is_some() && ui.input(|i| i.pointer.primary_down()) {
            lens(ui, wb, pos, map(pos), viewport);
        }
    }
    if ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary)) {
        wb.end_brush();
        if wb.moving_capture.is_some() || wb.eraser_last.take().is_some() {
            wb.sample_selected();
        }
        wb.moving_capture = None;
    }
}

fn lens(ui: &egui::Ui, wb: &SourceWorkbench, pos: Pos2, norm: [f32; 2], image: Rect) {
    let bounds = ui.clip_rect().intersect(image);
    let side = 96.0_f32.min(bounds.width()).min(bounds.height());
    if side < 36.0 {
        return;
    }
    let min =
        (pos + Vec2::new(20.0, -side - 20.0)).clamp(bounds.min, bounds.max - Vec2::splat(side));
    let rect = Rect::from_min_size(min, Vec2::splat(side));
    let half = Vec2::new(
        12.0 / wb.image.width() as f32,
        12.0 / wb.image.height() as f32,
    );
    let center = egui::pos2(norm[0], norm[1]);
    let uv = Rect::from_min_max(
        (center - half).max(Pos2::ZERO),
        (center + half).min(egui::pos2(1.0, 1.0)),
    );
    ui.painter()
        .rect_filled(rect.expand(3.0), 5.0, Color32::BLACK);
    ui.painter()
        .image(wb.texture.id(), rect, uv, Color32::WHITE);
    ui.painter().line_segment(
        [
            egui::pos2(rect.center().x, rect.top()),
            egui::pos2(rect.center().x, rect.bottom()),
        ],
        Stroke::new(1.0, Color32::WHITE),
    );
    ui.painter().line_segment(
        [
            egui::pos2(rect.left(), rect.center().y),
            egui::pos2(rect.right(), rect.center().y),
        ],
        Stroke::new(1.0, Color32::WHITE),
    );
    ui.painter().rect_stroke(
        rect.expand(3.0),
        5.0,
        Stroke::new(2.0, ACCENT),
        egui::StrokeKind::Inside,
    );
}

fn record(_ui: &egui::Ui, _name: &str, _rect: Rect) {
    #[cfg(test)]
    _ui.ctx()
        .data_mut(|d| d.insert_temp(egui::Id::new(("capture-test", _name)), _rect));
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use colorcraft_core::SourceAsset;

    fn bench(ctx: &egui::Context) -> SourceWorkbench {
        SourceWorkbench::new(
            ctx,
            SourceAsset {
                name: "Uma fonte com um nome muito longo que deve caber sem cortar as ferramentas"
                    .repeat(4),
                ..Default::default()
            },
            image::RgbaImage::from_pixel(400, 300, image::Rgba([32, 90, 180, 255])),
        )
    }
    fn frame(
        ctx: &egui::Context,
        wb: &mut SourceWorkbench,
        size: Vec2,
        events: Vec<egui::Event>,
    ) -> Outcome {
        let mut result = Outcome::default();
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
                events,
                focused: true,
                ..Default::default()
            },
            |ui| {
                result = show(ui, wb);
            },
        );
        output.textures_delta.clear();
        result
    }
    fn rect(ctx: &egui::Context, name: &str) -> Rect {
        ctx.data(|d| d.get_temp(egui::Id::new(("capture-test", name))))
            .unwrap()
    }
    fn input(ctx: &egui::Context, wb: &mut SourceWorkbench, pos: Pos2, pressed: bool) {
        frame(
            ctx,
            wb,
            Vec2::new(900.0, 640.0),
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
    #[test]
    fn stroke_eraser_removes_whole_hit_strokes_and_preserves_other_captures_and_pages() {
        let ctx = egui::Context::default();
        let mut wb = bench(&ctx);
        wb.start_brush([0.1, 0.4]);
        wb.continue_brush([0.9, 0.4]);
        wb.end_brush();
        wb.start_brush([0.1, 0.8]);
        wb.continue_brush([0.9, 0.8]);
        wb.end_brush();
        let survivor = wb.source.captures[0].strokes[1].clone();
        wb.new_capture([0.3, 0.4]);
        let other = wb.source.captures[1].clone();
        wb.selected = 0;
        crate::capture_mask::erase(&mut wb, [0.5, 0.2], [0.5, 0.6], 0.0);
        assert_eq!(wb.source.captures[0].strokes, vec![survivor.clone()]);
        assert_eq!(wb.source.captures[1], other);
        wb.page = 1;
        crate::capture_mask::erase(&mut wb, [0.5, 0.7], [0.5, 0.9], 0.0);
        assert_eq!(wb.source.captures[0].strokes, vec![survivor]);
        wb.page = 0;
        wb.tool = WorkbenchTool::Eraser;
        for _ in 0..3 {
            frame(&ctx, &mut wb, Vec2::new(900.0, 640.0), vec![]);
        }
        let image = rect(&ctx, "canvas");
        let point = image.min + image.size() * Vec2::new(0.5, 0.8);
        input(&ctx, &mut wb, point, true);
        input(&ctx, &mut wb, point, false);
        assert!(wb.source.captures[0].strokes.is_empty());
        assert_eq!(wb.source.captures[1], other);
        let apply = rect(&ctx, "apply").center();
        input(&ctx, &mut wb, apply, true);
        assert!(
            !frame(
                &ctx,
                &mut wb,
                Vec2::new(900.0, 640.0),
                vec![egui::Event::PointerButton {
                    pos: apply,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Default::default()
                }]
            )
            .apply
        );
    }

    #[test]
    fn deletion_uses_modal_blocks_canvas_and_can_be_cancelled_or_confirmed() {
        let ctx = egui::Context::default();
        let mut wb = bench(&ctx);
        wb.new_capture([0.2, 0.3]);
        wb.new_capture([0.7, 0.8]);
        wb.source.captures[0].color_id = Some("kept-palette-color".into());
        let original = wb.source.clone();
        wb.pending_delete = Some(0);
        for _ in 0..3 {
            frame(&ctx, &mut wb, Vec2::new(900.0, 640.0), vec![]);
        }
        let point = rect(&ctx, "canvas").left_top() + Vec2::splat(30.0);
        input(&ctx, &mut wb, point, true);
        input(&ctx, &mut wb, point, false);
        assert_eq!(wb.source, original);
        assert_eq!(wb.pending_delete, Some(0));
        let keep = rect(&ctx, "delete-keep").center();
        input(&ctx, &mut wb, keep, true);
        input(&ctx, &mut wb, keep, false);
        assert!(wb.pending_delete.is_none());
        assert_eq!(wb.source, original);
        wb.pending_delete = Some(0);
        for _ in 0..3 {
            frame(&ctx, &mut wb, Vec2::new(900.0, 640.0), vec![]);
        }
        let remove = rect(&ctx, "delete-remove").center();
        input(&ctx, &mut wb, remove, true);
        input(&ctx, &mut wb, remove, false);
        assert_eq!(wb.source.captures, vec![original.captures[1].clone()]);
        assert!(wb.dirty && wb.pending_delete.is_none());
        wb.pending_delete = Some(0);
        frame(
            &ctx,
            &mut wb,
            Vec2::new(900.0, 640.0),
            vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
        );
        assert!(wb.pending_delete.is_none());
        assert_eq!(wb.source.captures.len(), 1);
    }

    #[test]
    fn point_then_brush_keeps_both_captures_and_appends_strokes_only_to_mask() {
        let ctx = egui::Context::default();
        let mut wb = bench(&ctx);
        wb.new_capture([0.2, 0.3]);
        wb.source.captures[0].color_id = Some("existing-palette-color".into());
        let point = wb.source.captures[0].clone();
        wb.start_brush([0.6, 0.7]);
        wb.continue_brush([0.65, 0.72]);
        wb.end_brush();
        assert_eq!(wb.source.captures.len(), 2);
        assert_eq!(wb.source.captures[0], point);
        assert_ne!(wb.source.captures[0].id, wb.source.captures[1].id);
        assert_eq!(wb.source.captures[1].position, [0.6, 0.7]);
        wb.start_brush([0.7, 0.7]);
        wb.end_brush();
        assert_eq!(wb.source.captures.len(), 2);
        assert_eq!(wb.source.captures[1].strokes.len(), 2);
        wb.page = 1;
        wb.start_brush([0.5, 0.5]);
        wb.end_brush();
        assert_eq!(wb.source.captures.len(), 3);
        assert_eq!(wb.source.captures[2].page, 1);
        assert_eq!(wb.source.captures[0], point);
    }
    #[test]
    fn actual_canvas_clicks_select_and_drag_marker_without_duplicates() {
        let ctx = egui::Context::default();
        let mut wb = bench(&ctx);
        wb.new_capture([0.3, 0.4]);
        wb.new_capture([0.8, 0.8]);
        for _ in 0..3 {
            frame(&ctx, &mut wb, Vec2::new(900.0, 640.0), vec![]);
        }
        let image = rect(&ctx, "canvas");
        let pos = image.min + image.size() * Vec2::new(0.3, 0.4);
        for _ in 0..3 {
            input(&ctx, &mut wb, pos, true);
            input(&ctx, &mut wb, pos, false);
            assert_eq!(wb.selected, 0);
            assert_eq!(wb.source.captures.len(), 2);
        }
        let id = wb.source.captures[0].id.clone();
        let other = wb.source.captures[1].clone();
        input(&ctx, &mut wb, pos, true);
        let destination = pos + Vec2::new(45.0, 28.0);
        frame(
            &ctx,
            &mut wb,
            Vec2::new(900.0, 640.0),
            vec![egui::Event::PointerMoved(destination)],
        );
        input(&ctx, &mut wb, destination, false);
        assert_eq!(wb.source.captures.len(), 2);
        assert_eq!(wb.source.captures[0].id, id);
        assert!(wb.source.captures[0].position[0] > 0.35);
        assert_eq!(wb.source.captures[1], other);
        assert!(wb.moving_capture.is_none());
    }
    #[test]
    fn footer_is_visible_at_small_sizes_and_navigation_preserves_captures() {
        for dark in [true, false] {
            for size in [
                Vec2::new(680.0, 460.0),
                Vec2::new(1440.0, 920.0),
                Vec2::new(680.0 / 1.5, 460.0 / 1.5),
            ] {
                let ctx = egui::Context::default();
                crate::theme::apply(&ctx, dark, 1.0);
                let mut wb = bench(&ctx);
                wb.new_capture([0.3, 0.4]);
                for _ in 0..3 {
                    frame(&ctx, &mut wb, size, vec![]);
                }
                let bounds = Rect::from_min_size(Pos2::ZERO, size);
                assert!(
                    bounds.contains_rect(rect(&ctx, "apply")),
                    "Apply outside {size:?}"
                );
                assert!(bounds.contains_rect(rect(&ctx, "cancel")));
                assert!(rect(&ctx, "image-panel").width() > 50.0);
            }
        }
        let ctx = egui::Context::default();
        let mut wb = bench(&ctx);
        wb.new_capture([0.3, 0.4]);
        for _ in 0..3 {
            frame(&ctx, &mut wb, Vec2::new(900.0, 640.0), vec![]);
        }
        let original = wb.source.clone();
        wb.tool = WorkbenchTool::Hand;
        let center = rect(&ctx, "image-panel").center();
        input(&ctx, &mut wb, center, true);
        frame(
            &ctx,
            &mut wb,
            Vec2::new(900.0, 640.0),
            vec![egui::Event::PointerMoved(center + Vec2::new(40.0, 30.0))],
        );
        input(&ctx, &mut wb, center + Vec2::new(40.0, 30.0), false);
        assert!(wb.pan.length() > 0.0);
        wb.tool = WorkbenchTool::Zoom;
        input(&ctx, &mut wb, center, true);
        input(&ctx, &mut wb, center, false);
        assert!(wb.zoom > 1.0);
        assert_eq!(wb.source, original);
    }
    #[test]
    fn unaccepted_colour_warning_disables_apply_and_sampling() {
        let ctx = egui::Context::default();
        let mut wb = bench(&ctx);
        wb.color_warning = Some("Perfil ICC".into());
        for _ in 0..3 {
            frame(&ctx, &mut wb, Vec2::new(900.0, 640.0), vec![]);
        }
        let point = rect(&ctx, "image-panel").center();
        input(&ctx, &mut wb, point, true);
        input(&ctx, &mut wb, point, false);
        assert!(wb.source.captures.is_empty());
        let apply = rect(&ctx, "apply").center();
        input(&ctx, &mut wb, apply, true);
        assert!(
            !frame(
                &ctx,
                &mut wb,
                Vec2::new(900.0, 640.0),
                vec![egui::Event::PointerButton {
                    pos: apply,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Default::default()
                }]
            )
            .apply
        );
    }

    #[test]
    fn new_mask_button_creates_independent_mask_and_keeps_previous_strokes() {
        let ctx = egui::Context::default();
        let mut wb = bench(&ctx);
        wb.tool = WorkbenchTool::Brush;
        wb.start_brush([0.2, 0.2]);
        wb.continue_brush([0.25, 0.3]);
        wb.end_brush();
        let first = wb.source.captures[0].clone();
        for _ in 0..3 {
            frame(&ctx, &mut wb, Vec2::new(900.0, 640.0), vec![]);
        }
        let button = rect(&ctx, "new-capture").center();
        input(&ctx, &mut wb, button, true);
        input(&ctx, &mut wb, button, false);
        assert_eq!(wb.source.captures.len(), 2);
        assert_eq!(wb.selected, 1);
        assert_ne!(wb.source.captures[1].id, first.id);
        wb.start_brush([0.6, 0.6]);
        wb.continue_brush([0.7, 0.7]);
        wb.end_brush();
        assert_eq!(wb.source.captures.len(), 2);
        assert_eq!(wb.source.captures[0], first);
        assert_eq!(wb.source.captures[1].strokes.len(), 1);
    }
}
