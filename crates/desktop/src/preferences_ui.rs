use crate::{
    app::{Action, ColorCraftApp},
    i18n::tr,
    icons, theme,
};
use eframe::egui::{self, RichText};

impl ColorCraftApp {
    pub(crate) fn preferences_content(&mut self, ui: &mut egui::Ui) -> bool {
        theme::section(ui, "settings", "Preferências");
        ui.label(
            RichText::new(tr("Personalize sua interface e o salvamento das paletas."))
                .small()
                .weak(),
        );
        ui.add_space(10.0);
        let key = egui::Id::new("preferences-page");
        let mut page = ui.ctx().data(|d| d.get_temp::<usize>(key)).unwrap_or(0);
        ui.horizontal_wrapped(|ui| {
            for (index, icon, label) in [
                (0, "settings", "Interface"),
                (1, "save", "Arquivos e salvamento"),
                (2, "modes", "Workspace"),
            ] {
                if icons::text_toggle(ui, icon, label, page == index).clicked() {
                    page = index;
                }
            }
        });
        ui.ctx().data_mut(|d| d.insert_temp(key, page));
        ui.add_space(10.0);
        let mut prefs = self.settings.get().clone();
        let height = (ui.ctx().content_rect().height() - 190.0).clamp(80.0, 480.0);
        egui::ScrollArea::vertical().id_salt("preferences-scroll").max_height(height).auto_shrink([false, true]).show(ui, |ui| {
            let width = ui.available_width();
            theme::card(ui).show(ui, |ui| {
                ui.set_width((width - 24.0).max(80.0));
                match page {
                    0 => {
                        theme::section(ui, "settings", "Aparência");
                        ui.add_space(10.0);
                        ui.label(tr("Idioma da interface"));
                        egui::ComboBox::from_id_salt("interface-language").width(250.0_f32.min(ui.available_width()))
                            .selected_text(tr(if prefs.appearance.language == "en-US" { "Inglês" } else { "Português (Brasil)" }))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut prefs.appearance.language, "pt-BR".into(), tr("Português (Brasil)"));
                                ui.selectable_value(&mut prefs.appearance.language, "en-US".into(), tr("Inglês"));
                            });
                        ui.add_space(12.0);
                        ui.label(tr("Tema"));
                        ui.horizontal(|ui| {
                            if icons::text_toggle(ui, "moon", "Escuro", prefs.appearance.theme == "dark").clicked() { prefs.appearance.theme = "dark".into(); }
                            if icons::text_toggle(ui, "sun", "Claro", prefs.appearance.theme == "light").clicked() { prefs.appearance.theme = "light".into(); }
                        });
                        ui.add_space(12.0);
                        ui.label(tr("Escala da interface"));
                        ui.add(egui::Slider::new(&mut prefs.appearance.scale, 0.8..=1.5).max_decimals(2));
                    }
                    1 => {
                        theme::section(ui, "save", "Salvamento automático");
                        ui.add_space(8.0);
                        ui.checkbox(&mut prefs.general.auto_save_projects, tr("Salvar automaticamente projetos e atualizar a paleta pública"));
                        ui.label(RichText::new(tr("A cada 15 segundos, salva projetos já nomeados. Projetos novos ou importados aguardam Salvar/Salvar como.")).small().weak());
                        ui.add_space(8.0);
                        ui.checkbox(&mut prefs.general.autosave, tr("Manter rascunhos de recuperação automática"));
                        ui.add_space(12.0); ui.separator(); ui.add_space(8.0);
                        theme::section(ui, "folder", "Biblioteca local de paletas");
                        if let Some(paths) = &self.paths {
                            let path = paths.library.to_string_lossy().to_string();
                            ui.add(egui::Label::new(RichText::new(&path).monospace().small()).truncate()).on_hover_text(&path);
                            ui.label(RichText::new(tr("Paletas salvas são publicadas automaticamente. A biblioteca é independente de preferências e caches.")).small().weak());
                            if icons::text_button(ui, "folder", "Abrir pasta da biblioteca").clicked()
                                && let Err(e) = crate::dialogs::open_folder(&paths.library) { self.error = Some(e); }
                            if let Some(config) = &paths.config {
                                ui.add_space(10.0); ui.label(RichText::new(tr("Configuração:")).small().strong());
                                let text = config.to_string_lossy().to_string();
                                ui.add(egui::Label::new(RichText::new(&text).monospace().small()).truncate()).on_hover_text(text);
                            }
                        } else { ui.label(tr("Esta sessão usa preferências em memória.")); }
                    }
                    _ => {
                        theme::section(ui, "modes", "Disposição dos painéis");
                        ui.label(tr("Cada workspace guarda a disposição dos painéis. As paletas e seus dados não são alterados."));
                        ui.add_space(12.0);
                        if icons::text_button(ui, "refresh", "Restaurar disposição dos painéis").clicked() { self.issue(Action::ResetLayout); }
                    }
                }
            });
        });
        if prefs != *self.settings.get() {
            let language = prefs.appearance.language.clone();
            self.settings.update(|s| *s = prefs);
            crate::i18n::set_language(&language);
            self.apply_theme(ui.ctx());
        }
        ui.add_space(12.0);
        ui.separator();
        let mut close = false;
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), 28.0),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                close = ui.add(theme::primary("Concluir")).clicked();
            },
        );
        close
    }
}
