//! Composition workspace: recipes, live preview and a non-mutating palette picker.
use crate::{
    app::{ACCENT, Action, ColorCraftApp},
    dialogs::Dialog,
    icons, theme, wheel,
};
use colorcraft_core::{
    color, composition,
    engine::{Command, CompositionDraft, Replacement, Slot, composition_preview},
    *,
};
use eframe::egui::{self, Align2, Color32, FontId, Rect, RichText, Sense, Stroke, Vec2};
use std::collections::BTreeSet;

#[derive(Default)]
pub struct CompositionUi {
    pub info: Option<String>,
    pub removed: BTreeSet<String>,
    pub picked: bool,
}
pub struct Picking {
    pub draft: CompositionDraft,
    pub slot: usize,
    pub project_id: String,
}
fn title(kind: &str) -> &str {
    if kind == "gradient" {
        "Gradiente"
    } else {
        color::harmony_name(kind)
    }
}
fn description(kind: &str) -> &str {
    colorcraft_core::catalog::harmony(kind)
        .map(|t| t.info.description)
        .unwrap_or("Amostras intermediárias entre duas origens, no espaço escolhido.")
}
fn guidance(kind: &str) -> &str {
    match kind {
        "analogous" | "compound" => {
            "Comece em 30°. Reduza a abertura para aproximar as cores ou aumente até 60° para ampliar a variedade. A receita usa HSV; as cores originais são preservadas."
        }
        "rectangle" => {
            "Comece em 45° ou 60°. A abertura separa os pares complementares. Em 90° a disposição equivale ao quadrado."
        }
        "geometry" => {
            "Escolha o número de cores. A primeira fica na cor de origem; as demais ocupam vértices igualmente espaçados na roda, com separação de 360° dividida pelo total."
        }
        "gradient" => {
            "OKLab distribui as etapas de forma perceptual; OKLCH interpola matiz e cromaticidade; sRGB interpola os canais. A receita reduz cromaticidade ao gamut quando necessário. As intermediárias permanecem dependentes até serem destacadas."
        }
        "shades" | "mono" => {
            "A quantidade controla quantas amostras serão geradas. Use o resultado para hierarquia visual; os papéis e significados podem ser documentados depois, sem criar novas restrições."
        }
        _ => {
            "Escolha uma cor dominante e use as demais como apoio. A harmonia é uma relação HSV, não uma garantia de contraste de leitura; para isso, crie também um limite de contraste."
        }
    }
}
fn fitted(p: &Project, draft: &CompositionDraft, kind: &str) -> Option<CompositionDraft> {
    let mut d = draft.clone();
    d.kind = kind.into();
    d.replacements.clear();
    if kind == "gradient" && d.originals.len() == 1 {
        let id = d.originals.first()?;
        let other = p.colors().find(|i| &i.id != id)?;
        d.originals.push(other.id.clone());
    }
    for spread in [draft.spread, 15.0, 30.0, 45.0, 60.0, 90.0] {
        d.spread = spread;
        if composition_preview(p, &d).is_ok() {
            return Some(d);
        }
    }
    None
}
pub fn relation_label(p: &Project, id: &str) -> String {
    let names = |ids: Vec<&String>| {
        ids.into_iter()
            .filter_map(|id| p.item(id).map(Item::label))
            .collect::<Vec<_>>()
            .join(" · ")
    };
    if let Some(r) = p.links.iter().find(|r| r.id == id) {
        format!("Vínculo: {}", names(vec![&r.a, &r.b]))
    } else if let Some(r) = p.harmonies.iter().find(|r| r.id == id) {
        format!("{}: {}", title(&r.kind), names(r.ids.iter().collect()))
    } else if let Some(r) = p.gradients.iter().find(|r| r.id == id) {
        format!("Gradiente: {}", names(vec![&r.a, &r.b]))
    } else if let Some(r) = p.limits.iter().find(|r| r.id == id) {
        format!(
            "Contraste mínimo {:.2}:1: {}",
            r.minimum,
            names(vec![&r.a, &r.b])
        )
    } else {
        "Relação não identificada".into()
    }
}
impl ColorCraftApp {
    pub fn begin_composition_pick(&mut self, slot: usize) {
        if let Some(Dialog::Composition(draft)) = self.dialog.take() {
            self.picking = Some(Picking {
                draft,
                slot,
                project_id: self.project().map(|p| p.id.clone()).unwrap_or_default(),
            });
            self.error = None;
        }
    }
    pub fn cancel_composition_pick(&mut self) {
        if let Some(picking) = self.picking.take() {
            self.dialog = Some(Dialog::Composition(picking.draft));
        }
    }
    pub fn choose_composition_replacement(&mut self, id: &str) {
        let Some(picking) = &self.picking else {
            return;
        };
        let Some(p) = self.project() else {
            return;
        };
        if p.id != picking.project_id {
            return;
        }
        if !p.item(id).is_some_and(Item::is_color) {
            return;
        }
        if picking.draft.originals.iter().any(|i| i == id)
            || picking
                .draft
                .replacements
                .iter()
                .any(|(i, r)| *i != picking.slot && r.id == id)
        {
            self.error = Some("Esta cor já ocupa outra posição. Escolha uma cor diferente.".into());
            return;
        }
        if let Some(mut picking) = self.picking.take() {
            picking.draft.replacements.insert(
                picking.slot,
                Replacement {
                    id: id.into(),
                    unlock: false,
                    detach: false,
                },
            );
            self.composition_ui.picked = true;
            self.dialog = Some(Dialog::Composition(picking.draft));
            self.error = None;
        }
    }
    pub fn composition_modal(&mut self, ctx: &egui::Context, draft: &mut CompositionDraft) -> bool {
        let p = self.project().cloned().unwrap_or_default();
        let before = draft.clone();
        let removed = self.composition_ui.removed.clone();
        let mut keep = true;
        let modal_response=egui::Modal::new(egui::Id::new("composition-workspace")).show(ctx,|ui|{
            let bounds=ctx.content_rect();
            let width=(bounds.width()-48.0).clamp(300.0,1160.0);
            ui.set_width(width);
            ui.horizontal(|ui|{
                ui.heading(crate::i18n::tr("Explorar composições"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{
                    if icons::button(ui,"close","Cancelar composição").clicked(){keep=false;}
                });
            });
            ui.label(RichText::new(crate::i18n::tr("Escolha uma receita, ajuste seus parâmetros e confira as cores antes de aplicar.")).weak());
            ui.add_space(10.0);
            let height=(bounds.height()-205.0).clamp(180.0,650.0);
            if width>=640.0 {
                let left=(width*0.31).clamp(220.0,340.0);
                ui.horizontal_top(|ui|{
                    ui.allocate_ui_with_layout(Vec2::new(left,height),egui::Layout::top_down(egui::Align::Min),|ui|{
                        ui.set_width(left);
                        egui::ScrollArea::vertical().id_salt("composition-recipes-scroll").max_height(height).min_scrolled_height(height).show(ui,|ui|self.recipe_cards(ui,&p,draft));
                    });
                    let (separator,_) = ui.allocate_exact_size(Vec2::new(1.0,height),Sense::hover());
                    ui.painter().line_segment([separator.center_top(),separator.center_bottom()],Stroke::new(1.0,ui.visuals().widgets.noninteractive.bg_stroke.color));
                    ui.allocate_ui_with_layout(Vec2::new((width-left-27.0).max(100.0),height),egui::Layout::top_down(egui::Align::Min),|ui|{
                        ui.set_width((width-left-27.0).max(100.0));
                        egui::ScrollArea::vertical().id_salt("composition-preview-scroll").max_height(height).min_scrolled_height(height).show(ui,|ui|self.composition_controls(ui,&p,draft));
                    });
                });
            } else {
                egui::ScrollArea::vertical().id_salt("composition-small-scroll").max_height(height).show(ui,|ui|{
                    self.recipe_cards(ui,&p,draft); ui.separator(); self.composition_controls(ui,&p,draft);
                });
            }
            ui.add_space(8.0);ui.separator();
            let analysis=composition::analyze(&p,draft,&self.composition_ui.removed);
            ui.horizontal(|ui|{
                if ui.button(crate::i18n::tr("Cancelar")).clicked(){keep=false;}
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{
                    if ui.add_enabled(analysis.issue.is_none(),theme::primary("Aplicar composição")).clicked(){
                        let mut commands:Vec<_>=self.composition_ui.removed.iter().cloned().map(Command::RemoveRelation).collect();
                        commands.push(Command::Compose(draft.clone()));self.issue(Action::Confirm(Command::Batch(commands)));
                    }
                });
            });
        });
        #[cfg(test)]
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("test-composition-modal"),
                modal_response.response.rect,
            )
        });
        let _ = modal_response;
        let picked = std::mem::take(&mut self.composition_ui.picked);
        let interacted = ctx.input(|i| {
            i.pointer.any_down()
                || i.pointer.any_released()
                || i.raw.events.iter().any(|e| {
                    matches!(
                        e,
                        egui::Event::Text(_)
                            | egui::Event::Paste(_)
                            | egui::Event::Key { pressed: true, .. }
                    )
                })
        });
        if (((before != *draft || removed != self.composition_ui.removed) && interacted) || picked)
            && let Some(diagnostic) = composition::analyze(&p, draft, &self.composition_ui.removed)
                .issue
                .and_then(|i| i.diagnostic)
            && let Some(doc) = self.doc_mut()
        {
            doc.conflicts.record(&p, *diagnostic, true);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            keep = false;
        }
        keep
    }
    fn recipe_cards(&mut self, ui: &mut egui::Ui, p: &Project, draft: &mut CompositionDraft) {
        ui.label(
            RichText::new(crate::i18n::tr("RECEITA"))
                .small()
                .strong()
                .weak(),
        );
        for kind in colorcraft_core::catalog::HARMONY_TYPES
            .iter()
            .map(|t| t.info.id)
            .chain(std::iter::once("gradient"))
        {
            ui.push_id(kind,|ui|{
                let fit=fitted(p,draft,kind);
                let active=draft.kind==kind;
                let (rect,response)=ui.allocate_exact_size(Vec2::new(ui.available_width(),103.0),if fit.is_some(){Sense::click()}else{Sense::hover()});
                let fill=if active {ui.visuals().selection.bg_fill}else if response.hovered(){ui.visuals().widgets.hovered.bg_fill}else{ui.visuals().extreme_bg_color};
                ui.painter().rect_filled(rect,7,fill);
                ui.painter().rect_stroke(rect,7,Stroke::new(if active{1.5}else{1.0},if active{ACCENT}else{ui.visuals().widgets.noninteractive.bg_stroke.color}),egui::StrokeKind::Inside);
                let info_rect=Rect::from_min_size(rect.right_top()+Vec2::new(-31.0,4.0),Vec2::splat(25.0));
                let over_info=ui.input(|i|i.pointer.hover_pos().is_some_and(|p|info_rect.contains(p)));
                if icons::at(ui,info_rect,"info",&format!("Sobre {}",title(kind)),false).clicked(){self.composition_ui.info=if self.composition_ui.info.as_deref()==Some(kind){None}else{Some(kind.into())};}
                response.widget_info(||egui::WidgetInfo::selected(egui::WidgetType::RadioButton,fit.is_some(),active,title(kind)));
                let slots=fit.as_ref().and_then(|d|composition_preview(p,d).ok()).map(|(s,_)|s).unwrap_or_default();
                let center=rect.min+Vec2::new(33.0,35.0);
                ui.painter().circle_stroke(center,20.0,Stroke::new(1.0,ui.visuals().weak_text_color()));
                for slot in &slots {let h=color::hsv(slot.rgb)[0].to_radians() as f32;let point=center+Vec2::new(h.cos(),-h.sin())*20.0;ui.painter().line_segment([center,point],Stroke::new(0.8,ui.visuals().weak_text_color()));ui.painter().circle_filled(point,3.5,wheel::rgb(slot.rgb));}
                let copy=Rect::from_min_max(rect.min+Vec2::new(65.0,9.0),rect.max-Vec2::new(28.0,25.0));
                let mut copy_ui=ui.new_child(egui::UiBuilder::new().max_rect(copy)); let ui=&mut copy_ui; {
                    ui.label(RichText::new(crate::i18n::tr(title(kind))).strong().color(if active{ACCENT}else{ui.visuals().text_color()}));
                    ui.label(RichText::new(crate::i18n::tr(description(kind))).small().weak());
                }
                let mini=Rect::from_min_max(rect.left_bottom()+Vec2::new(9.0,-15.0),rect.right_bottom()-Vec2::new(9.0,7.0));
                for (i,slot) in slots.iter().enumerate(){let w=mini.width()/slots.len().max(1) as f32;ui.painter().rect_filled(Rect::from_min_size(mini.min+Vec2::new(i as f32*w,0.0),Vec2::new(w+0.5,mini.height())),0,wheel::rgb(slot.rgb));}
                if fit.is_none(){response.clone().on_hover_text(crate::i18n::tr("Estas originais não cabem nesta receita. Selecione outras cores de origem para utilizá-la."));}
                if response.clicked()&&!over_info&&let Some(fit)=fit { *draft=fit; self.composition_ui.removed.clear(); }
            });
        }
    }
    fn composition_controls(
        &mut self,
        ui: &mut egui::Ui,
        p: &Project,
        draft: &mut CompositionDraft,
    ) {
        ui.heading(crate::i18n::tr(title(&draft.kind)));
        ui.label(RichText::new(crate::i18n::tr(description(&draft.kind))).weak());
        if let Some(info) = &self.composition_ui.info {
            ui.group(|ui| {
                ui.label(RichText::new(crate::i18n::tr(format!("Sobre {}", title(info)))).strong());
                ui.label(crate::i18n::tr(guidance(info)));
            });
        }
        ui.add_space(8.0);
        ui.label(
            RichText::new(crate::i18n::tr("ORIGINAIS PRESERVADAS"))
                .small()
                .strong()
                .weak(),
        );
        ui.horizontal_wrapped(|ui| {
            for id in &draft.originals {
                if let Some(item) = p.item(id) {
                    ui.group(|ui| {
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::new(30.0, 22.0), Sense::hover());
                        ui.painter().rect_filled(rect, 4, wheel::rgb(item.rgb));
                        ui.label(item.label());
                    });
                }
            }
        });
        ui.add_space(8.0);
        if let Some(parameter) =
            colorcraft_core::catalog::harmony(&draft.kind).and_then(|t| t.spread)
        {
            ui.label(crate::i18n::tr(parameter.label));
            ui.add(
                egui::Slider::new(&mut draft.spread, parameter.min..=parameter.max)
                    .clamping(egui::SliderClamping::Edits)
                    .suffix(parameter.suffix)
                    .max_decimals(2),
            );
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
            theme::chips(ui, &mut draft.spread, &choices);
        }
        if draft.kind == "gradient"
            || colorcraft_core::catalog::harmony(&draft.kind).is_some_and(|t| t.count.is_some())
        {
            let parameter = colorcraft_core::catalog::harmony(&draft.kind).and_then(|t| t.count);
            ui.label(if draft.kind == "gradient" {
                crate::i18n::tr("Cores intermediárias")
            } else {
                crate::i18n::tr(parameter.map_or("Quantidade", |p| p.label))
            });
            let old = draft.count;
            let min = parameter.map_or(1, |p| p.min as usize);
            let max = parameter.map_or(32, |p| p.max as usize);
            ui.add(
                egui::DragValue::new(&mut draft.count)
                    .range(min..=max)
                    .clamp_existing_to_range(false),
            );
            let suggestions =
                parameter.map_or(&[1.0, 2.0, 3.0, 5.0, 7.0, 9.0][..], |p| p.suggestions);
            let labels: Vec<_> = suggestions.iter().map(|n| n.to_string()).collect();
            let choices: Vec<_> = suggestions
                .iter()
                .zip(&labels)
                .map(|(n, l)| (*n as usize, l.as_str()))
                .collect();
            theme::chips(ui, &mut draft.count, &choices);
            if old != draft.count {
                draft.replacements.clear();
            }
        }
        if draft.kind == "gradient" {
            ui.label(crate::i18n::tr("Interpolação"));
            theme::chips(
                ui,
                &mut draft.space,
                &[
                    ("oklab".into(), "OKLab"),
                    ("oklch".into(), "OKLCH"),
                    ("srgb".into(), "sRGB"),
                ],
            );
            ui.label(RichText::new(crate::i18n::tr("Redução de cromaticidade ao gamut sRGB quando necessária; as originais permanecem intactas.")).small().weak());
        }
        let preview = composition_preview(p, draft);
        if let Ok((slots, _)) = &preview {
            // A recipe change must not leave a replacement pointing at an original or stale slot.
            draft
                .replacements
                .retain(|index, _| slots.get(*index).is_some_and(|s| s.original.is_none()));
            ui.add_space(12.0);
            ui.separator();
            ui.label(
                RichText::new(crate::i18n::tr("PRÉVIA"))
                    .small()
                    .strong()
                    .weak(),
            );
            let columns = if ui.available_width() >= 380.0 { 2 } else { 1 };
            let width =
                ((ui.available_width() - (columns - 1) as f32 * 10.0) / columns as f32).max(120.0);
            for (row_index, row) in slots.chunks(columns).enumerate() {
                ui.horizontal_top(|ui| {
                    for (column, slot) in row.iter().enumerate() {
                        let index = row_index * columns + column;
                        ui.push_id(index, |ui| {
                            self.composition_slot(ui, p, draft, index, slot, width)
                        });
                    }
                });
                ui.add_space(3.0);
            }
        }
        let analysis = composition::analyze(p, draft, &self.composition_ui.removed);
        ui.label(
            RichText::new(format!(
                "{} originais · {} novas · {} substituições",
                analysis.original_count, analysis.new_count, analysis.replacement_count
            ))
            .small(),
        );
        if let Some(issue) = analysis.issue {
            ui.group(|ui| {
                ui.label(RichText::new(crate::i18n::tr(issue.message)).color(ACCENT));
                for id in issue.relation_ids {
                    if !p.links.iter().any(|r| r.id == id)
                        && !p.harmonies.iter().any(|r| r.id == id)
                        && !p.gradients.iter().any(|r| r.id == id)
                        && !p.limits.iter().any(|r| r.id == id)
                    {
                        continue;
                    }
                    let mut removed = self.composition_ui.removed.contains(&id);
                    if ui
                        .checkbox(
                            &mut removed,
                            crate::i18n::tr(format!(
                                "Remover ao aplicar: {}",
                                relation_label(p, &id)
                            )),
                        )
                        .changed()
                    {
                        if removed {
                            self.composition_ui.removed.insert(id);
                        } else {
                            self.composition_ui.removed.remove(&id);
                        }
                    }
                }
            });
        }
        for id in self.composition_ui.removed.clone() {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(crate::i18n::tr(format!(
                        "Será removido: {}",
                        relation_label(p, &id)
                    )))
                    .small()
                    .color(ACCENT),
                );
                if icons::button(ui, "undo", "Manter esta relação").clicked() {
                    self.composition_ui.removed.remove(&id);
                }
            });
        }
    }
    fn composition_slot(
        &mut self,
        ui: &mut egui::Ui,
        p: &Project,
        draft: &mut CompositionDraft,
        index: usize,
        slot: &Slot,
        width: f32,
    ) {
        let mut replacement = draft.replacements.get(&index).cloned().unwrap_or_default();
        let target = p.item(&replacement.id);
        let locked = target.is_some_and(|i| i.locked || i.direct_locked);
        let dependent = target.is_some_and(|i| {
            p.gradients
                .iter()
                .any(|g| g.ids.contains(&i.id) && !g.detached.contains(&i.id))
        });
        let height = 160.0 + if locked { 25.0 } else { 0.0 } + if dependent { 25.0 } else { 0.0 };
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        ui.painter()
            .rect_filled(rect, 7, ui.visuals().extreme_bg_color);
        let color_rect = Rect::from_min_size(rect.min, Vec2::new(width, 98.0));
        ui.painter()
            .rect_filled(color_rect, 7, wheel::rgb(slot.rgb));
        let fg = if color::luminance(slot.rgb) > 0.179 {
            Color32::from_rgb(24, 25, 29)
        } else {
            Color32::WHITE
        };
        let label = if slot.original.is_some() {
            "ORIGINAL"
        } else if target.is_some() {
            "SUBSTITUI"
        } else {
            "NOVA"
        };
        ui.painter().text(
            color_rect.min + Vec2::new(10.0, 11.0),
            Align2::LEFT_TOP,
            label,
            FontId::proportional(10.0),
            fg,
        );
        ui.painter().text(
            color_rect.left_bottom() + Vec2::new(10.0, -14.0),
            Align2::LEFT_BOTTOM,
            color::hex(slot.rgb),
            FontId::monospace(14.0),
            fg,
        );
        let item = slot.original.as_ref().and_then(|id| p.item(id)).or(target);
        let name = item
            .map(Item::label)
            .unwrap_or_else(|| "Criar nova cor".into());
        let limit = ((width - 20.0) / 6.7).max(5.0) as usize;
        let shown = if name.chars().count() > limit {
            format!(
                "{}…",
                name.chars()
                    .take(limit.saturating_sub(1))
                    .collect::<String>()
            )
        } else {
            name
        };
        ui.painter().text(
            rect.min + Vec2::new(10.0, 111.0),
            Align2::LEFT_TOP,
            shown,
            FontId::proportional(12.0),
            ui.visuals().text_color(),
        );
        if slot.original.is_none() {
            if ui
                .put(
                    Rect::from_min_size(
                        rect.min + Vec2::new(8.0, 130.0),
                        Vec2::new((width - 47.0).max(80.0), 25.0),
                    ),
                    egui::Button::new(crate::i18n::tr("Substituir…")),
                )
                .clicked()
            {
                self.issue(Action::BeginPick(index));
            }
            if target.is_some()
                && icons::at(
                    ui,
                    Rect::from_min_size(
                        rect.right_top() + Vec2::new(-34.0, 128.0),
                        Vec2::splat(27.0),
                    ),
                    "close",
                    "Criar nova cor nesta posição",
                    false,
                )
                .clicked()
            {
                draft.replacements.remove(&index);
                replacement = Replacement::default();
            }
            let mut y = 160.0;
            if locked {
                ui.place(
                    Rect::from_min_size(
                        rect.min + Vec2::new(8.0, y),
                        Vec2::new(width - 16.0, 24.0),
                    ),
                    egui::Checkbox::new(&mut replacement.unlock, "Liberar bloqueios ao aplicar"),
                );
                y += 25.0;
            }
            if dependent {
                ui.place(
                    Rect::from_min_size(
                        rect.min + Vec2::new(8.0, y),
                        Vec2::new(width - 16.0, 24.0),
                    ),
                    egui::Checkbox::new(&mut replacement.detach, "Destacar do gradiente"),
                );
            }
            if !replacement.id.is_empty() {
                draft.replacements.insert(index, replacement);
            }
        }
    }
}
