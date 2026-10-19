use crate::{
    app::{ACCENT, Action, ColorCraftApp},
    wheel,
};
use colorcraft_core::{
    engine::{Command, CompositionDraft},
    export::{self, Format},
    *,
};
use eframe::egui::{self, RichText, Vec2};

pub enum Dialog {
    Workspaces {
        new_name: String,
        rename_name: String,
        selected: String,
    },
    Preferences,
    Guide,
    About,
    Credits,
    Project {
        name: String,
        description: String,
    },
    Export {
        format: Format,
        appendix: bool,
        accepted: bool,
    },
    Delete(Vec<String>),
    RemoveSource {
        id: String,
        linked_colors: usize,
    },
    RenameSource {
        id: String,
        name: String,
    },
    UpdateSource {
        id: String,
        path: String,
    },
    SourceColorUpdate {
        id: String,
        path: String,
        warning: String,
    },
    UnlinkCapture {
        item_id: String,
        source_id: String,
        capture_id: String,
    },
    Move {
        ids: Vec<String>,
        parent: Option<String>,
    },
    Close {
        index: usize,
        exit: bool,
    },
    Link {
        participants: Vec<String>,
        a: String,
        b: String,
        channels: [bool; 6],
        kind: usize,
        minimum: f64,
        parametric_type: String,
        limit_type: String,
        convert: bool,
        relation_kind: String,
        note: String,
    },
    Composition(CompositionDraft),
    Conversion {
        id: String,
        target: String,
        assign: bool,
    },
    Gamut {
        id: String,
        original: [f64; 3],
        mapped: [f64; 3],
    },
    Documentation {
        roles: Vec<Role>,
        modes: Vec<Mode>,
        relations: Vec<Documentary>,
    },
}
impl Dialog {
    pub fn is_command_dialog(&self) -> bool {
        matches!(
            self,
            Self::Project { .. }
                | Self::Delete(_)
                | Self::RemoveSource { .. }
                | Self::RenameSource { .. }
                | Self::UpdateSource { .. }
                | Self::SourceColorUpdate { .. }
                | Self::UnlinkCapture { .. }
                | Self::Move { .. }
                | Self::Link { .. }
                | Self::Composition(_)
                | Self::Conversion { .. }
                | Self::Gamut { .. }
                | Self::Documentation { .. }
        )
    }
}
pub fn color_combo(
    ui: &mut egui::Ui,
    p: &Project,
    id: impl std::hash::Hash + std::fmt::Debug,
    value: &mut String,
    folders: bool,
) {
    let text = p
        .item(value)
        .map(Item::label)
        .unwrap_or_else(|| crate::i18n::tr("Escolher…"));
    egui::ComboBox::from_id_salt(id)
        .selected_text(text)
        .width(210.0)
        .show_ui(ui, |ui| {
            for item in p.items.iter().filter(|i| folders || i.is_color()) {
                ui.selectable_value(value, item.id.clone(), item.label());
            }
        });
}
impl ColorCraftApp {
    pub fn show_dialogs(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.dialog.take() else {
            return;
        };
        if let Dialog::Composition(mut draft) = dialog {
            if self.composition_modal(ctx, &mut draft) {
                self.dialog = Some(Dialog::Composition(draft));
            }
            return;
        }
        let p = self.project().cloned().unwrap_or_default();
        let mut close = false;
        let mut reopen = true;
        egui::Modal::new(egui::Id::new("colorcraft-dialog")).frame(egui::Frame::window(&ctx.global_style()).inner_margin(if matches!(dialog, Dialog::Link{..}) { 20 } else { 12 })).show(ctx,|ui|{let width=(ctx.content_rect().width()-64.0).clamp(240.0, if matches!(dialog,Dialog::Preferences){620.0}else if matches!(dialog,Dialog::Link{..}){740.0}else{820.0});ui.set_width(width);match &mut dialog{
  Dialog::Workspaces { new_name, rename_name, selected } => {
    ui.heading(crate::i18n::tr("Workspaces"));
    ui.label(crate::i18n::tr("Cada workspace guarda a disposição dos painéis. As paletas e seus dados não são alterados."));
    ui.add_space(8.0);
    let entries = self.workspaces.presets.clone();
    let selected_name = entries.iter().find(|w| w.id == *selected).map(|w| if w.builtin { crate::i18n::tr(&w.name) } else { w.name.clone() }).unwrap_or_else(|| crate::i18n::tr("Escolha um workspace"));
    egui::ComboBox::from_id_salt("workspace-manager-select").selected_text(selected_name).width(420.0).show_ui(ui, |ui| {
        for workspace in &entries {
            let display_name = if workspace.builtin { crate::i18n::tr(&workspace.name) } else { workspace.name.clone() };
            if ui.selectable_label(*selected == workspace.id, display_name).clicked() {
                *selected = workspace.id.clone();
                self.activate_workspace(&workspace.id);
                *rename_name = workspace.name.clone();
            }
        }
    });
    let builtin = entries.iter().find(|w| w.id == *selected).is_some_and(|w| w.builtin);
    ui.separator();
    ui.label(crate::i18n::tr("Salvar a disposição atual como um novo workspace"));
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(new_name).hint_text(crate::i18n::tr("Nome do workspace")).desired_width(290.0));
        if ui.button(crate::i18n::tr("Criar a partir da disposição atual")).clicked() {
            self.create_workspace(new_name);
            if let Some(workspace) = self.workspaces.presets.iter().find(|w| w.id == self.workspaces.active) {
                *selected = workspace.id.clone();
                *rename_name = workspace.name.clone();
                *new_name = String::new();
            }
        }
    });
    ui.separator();
    ui.label(crate::i18n::tr("Workspace selecionado"));
    ui.horizontal(|ui| {
        ui.add_enabled_ui(!builtin, |ui| {
            ui.add(egui::TextEdit::singleline(rename_name).desired_width(290.0));
            if ui.button(crate::i18n::tr("Renomear")).clicked() { self.rename_workspace(selected, rename_name); }
        });
        if builtin && ui.button(crate::i18n::tr("Redefinir")).clicked()
            && let Some(layout) = self.workspaces.reset(selected)
        {
            self.layout = layout;
            self.layout_dirty = true;
        }
        if !builtin && ui.button(crate::i18n::tr("Excluir")).clicked() {
            self.delete_workspace(selected);
            *selected = self.workspaces.active.clone();
            if let Some(workspace) = self.workspaces.presets.iter().find(|w| w.id == *selected) {
                *rename_name = workspace.name.clone();
            }
        }
    });
    ui.separator();
    if ui.button(crate::i18n::tr("Concluir")).clicked() { close = true; }
  },
  Dialog::About=>{ui.horizontal(|ui|{if let Some(logo)=&self.logo{ui.image((logo.id(),Vec2::splat(96.0)));}ui.vertical(|ui|{ui.heading(crate::i18n::tr("ColorCraft"));ui.label(crate::i18n::tr(format!("Versão {} · Windows standalone",env!("CARGO_PKG_VERSION"))));ui.label(crate::i18n::tr("Paletas com valores, relações e significado."));});});ui.add_space(12.0);ui.label(crate::i18n::tr("Concepção, ícone e direção do produto: Eduardo."));ui.label(crate::i18n::tr("Aplicativo independente, desenvolvido em Rust. Integração com os aplicativos Craft é uma direção de projeto; esta versão não é um produto oficial ArtCraft."));ui.label(crate::i18n::tr("Bibliotecas Craft utilizadas com seus créditos e licenças preservados."));ui.add_space(12.0);
if ui.button(crate::i18n::tr("Fechar")).clicked(){close=true;}},
  Dialog::Credits=>{ui.heading(crate::i18n::tr("Licenças e créditos"));ui.label(crate::i18n::tr("Código ColorCraft: MIT OU Apache-2.0."));ui.label(crate::i18n::tr("craft-ui, craft-cms, craft-settings e craft-atomic: equipe ArtCraft e contribuidores, MIT OU Apache-2.0."));ui.label(crate::i18n::tr("Interface: egui/eframe; diálogos nativos: rfd; imagens: image. As dependências mantêm seus próprios avisos e licenças, incluídos no pacote."));ui.label(crate::i18n::tr("Ícone: fornecido por Eduardo para uso no ColorCraft. A licença geral de reutilização da imagem não foi declarada."));ui.label(crate::i18n::tr("Matemática: especificações W3C CSS Color, WCAG 2.2, ICC e publicação original de OKLab. Exemplos e implementação de relações próprios, portados do Colorator."));
if ui.button(crate::i18n::tr("Fechar")).clicked(){close=true;}},
  Dialog::Guide=>{ui.heading(crate::i18n::tr("Comece pelo ColorCraft"));for text in ["1. Crie uma paleta na Home ou importe o JSON do Colorator.","2. Organize cores e pastas em cards ou árvore. Ctrl seleciona várias cores; arraste para mover e ordenar.","3. Selecione uma cor para usar a roda e os canais. Um gesto é uma etapa de desfazer.","4. Em Vínculos, mantenha razões RGB/HSV, crie harmonias, gradientes e contraste mínimo WCAG 2.2.","5. Gradientes controlam as intermediárias até você destacá-las. Bloqueios e limites nunca descartam as relações.","6. Em Detalhes, registre nomes, tags, campos próprios e uso. Papéis e modos associam significados às cores sem alterar valores.","7. Perfis e valores nativos são preservados. Trocar espaço ou perfil exige confirmar a comparação; a roda usa sRGB.","8. Salvar publica a paleta na biblioteca local. Exportar oferece um snapshot para consumo ou formatos de intercâmbio com perdas identificadas.","9. Arraste os títulos dos painéis para reorganizar, agrupar ou desencaixar. Janela restaura o espaço padrão."]{ui.label(crate::i18n::tr(text));ui.add_space(5.0);}ui.separator();ui.label(crate::i18n::tr("Ctrl+N · nova  |  Ctrl+O · abrir  |  Ctrl+S · salvar  |  Ctrl+Shift+S · salvar cópia"));ui.label(crate::i18n::tr("Ctrl+Z / Ctrl+Shift+Z · desfazer/refazer  |  Ctrl+C/X/V · copiar/recortar/colar  |  Ctrl+D · duplicar"));ui.label(crate::i18n::tr("Rascunhos de recuperação são gravados separadamente. Eles não substituem o salvamento nem seus backups."));
if ui.button(crate::i18n::tr("Fechar")).clicked(){close=true;}},
  Dialog::Preferences=>{close=self.preferences_content(ui);},
  Dialog::Project{name,description}=>{ui.heading(crate::i18n::tr("Detalhes do projeto"));ui.label(crate::i18n::tr("Nome"));ui.add(egui::TextEdit::singleline(name).desired_width(460.0));ui.label(crate::i18n::tr("Descrição"));ui.add(egui::TextEdit::multiline(description).desired_rows(4).desired_width(460.0));ui.separator();ui.horizontal(|ui|{if ui.button(crate::i18n::tr("Cancelar")).clicked(){close=true}
if ui.button(crate::i18n::tr("Aplicar")).clicked(){self.issue(Action::Confirm(Command::ProjectDetails{name:name.clone(),description:description.clone()}));}});},
  Dialog::Delete(ids)=>{ui.heading(crate::i18n::tr("Excluir itens?"));let all=p.descendants(ids);ui.label(crate::i18n::tr(format!("{} itens, incluindo conteúdo das pastas selecionadas. Relações que dependem desses itens também serão removidas.",all.len())));ui.label(crate::i18n::tr("Você poderá desfazer esta operação."));ui.separator();ui.horizontal(|ui|{if ui.button(crate::i18n::tr("Cancelar")).clicked(){close=true;}
if ui.button(crate::i18n::tr("Excluir itens")).clicked(){self.issue(Action::Confirm(Command::Delete(ids.clone())));}});},
  Dialog::RemoveSource{id,linked_colors}=>{ui.heading(crate::i18n::tr("Remover fonte?"));ui.label(crate::i18n::tr(format!("A fonte e suas capturas serão removidas. {linked_colors} cor(es) permanecerão na raiz e serão desvinculadas.")));ui.separator();ui.horizontal(|ui|{if ui.button(crate::i18n::tr("Cancelar")).clicked(){close=true;}
if ui.button(crate::i18n::tr("Remover fonte e preservar cores")).clicked(){self.issue(Action::Confirm(Command::RemoveSource(id.clone())));}});},
  Dialog::RenameSource{id,name}=>{ui.heading(crate::i18n::tr("Renomear fonte"));ui.label(crate::i18n::tr("O nome identifica o arquivo no painel Fontes; o caminho vinculado não muda."));ui.add(egui::TextEdit::singleline(name).desired_width(460.0));ui.separator();ui.horizontal(|ui|{if ui.button(crate::i18n::tr("Cancelar")).clicked(){close=true;}
if ui.add_enabled(!name.trim().is_empty(),egui::Button::new(crate::i18n::tr("Aplicar nome"))).clicked() && let Some(mut source)=p.sources.iter().find(|s|s.id==*id).cloned(){source.name=name.trim().to_string();self.issue(Action::Confirm(Command::ApplySource{source,colors_to_add:vec![],color_values:Default::default()}));}});},
  Dialog::UpdateSource{id,path}=>{ui.heading(crate::i18n::tr("Atualizar capturas da fonte?"));ui.label(crate::i18n::tr(format!("O arquivo {} será renderizado novamente. As cores vinculadas serão recalculadas pela posição e método salvos; nada muda até confirmar.",path)));ui.separator();ui.horizontal(|ui|{if ui.button(crate::i18n::tr("Cancelar")).clicked(){close=true;}
if ui.button(crate::i18n::tr("Atualizar e recalcular")).clicked(){self.queue_update_source(id,path);if self.dialog.is_some(){reopen=false;}}});},
  Dialog::SourceColorUpdate{id,path,warning}=>{ui.heading(crate::i18n::tr("Atualizar usando as cores da prévia?"));ui.label(crate::i18n::tr(warning.as_str()));ui.label(crate::i18n::tr("Somente a confirmação abaixo recalcula as cores vinculadas. Você poderá desfazer toda a atualização."));ui.horizontal_wrapped(|ui|{if ui.button(crate::i18n::tr("Cancelar para converter uma cópia")).clicked(){close=true;}
if ui.button(crate::i18n::tr("Usar prévia PDFium e atualizar")).clicked(){self.queue_update_source_with_preview(id,path);}});},
  Dialog::UnlinkCapture{item_id,source_id,capture_id}=>{let name=p.item(item_id).map(Item::label).unwrap_or_else(||"Cor".into());ui.heading(crate::i18n::tr("Desvincular cor da fonte?"));ui.label(crate::i18n::tr(format!("“{name}” está bloqueada porque acompanha uma captura. O valor atual será preservado e a cor ficará livre para edição.")));ui.separator();ui.horizontal(|ui|{if ui.button(crate::i18n::tr("Cancelar")).clicked(){close=true;}
if ui.button(crate::i18n::tr("Desvincular e desbloquear")).clicked(){self.issue(Action::Confirm(Command::UnlinkCapture{source_id:source_id.clone(),capture_id:capture_id.clone()}));}});},
  Dialog::Move{ids,parent}=>{ui.heading(crate::i18n::tr("Mover para pasta"));egui::ComboBox::from_id_salt("move-destination").selected_text(parent.as_deref().and_then(|id|p.item(id)).map(Item::label).unwrap_or("Raiz da paleta".into())).show_ui(ui,|ui|{ui.selectable_value(parent,None,crate::i18n::tr("Raiz da paleta"));let all=p.descendants(ids);for folder in p.items.iter().filter(|i|i.kind=="folder"&&!all.contains(&i.id)){ui.selectable_value(parent,Some(folder.id.clone()),folder.label());}});ui.separator();ui.horizontal(|ui|{if ui.button(crate::i18n::tr("Cancelar")).clicked(){close=true;}
if ui.button(crate::i18n::tr("Mover")).clicked(){self.issue(Action::Confirm(Command::Move{ids:ids.clone(),parent:parent.clone()}));}});},
  Dialog::Close{index,exit}=>{ui.heading(crate::i18n::tr("Salvar alterações?"));let name=self.docs.get(*index).map(|d|d.engine.project.name.as_str()).unwrap_or("paleta");ui.label(crate::i18n::tr(format!("“{name}” tem alterações não salvas.")));ui.separator();ui.horizontal(|ui|{if ui.button(crate::i18n::tr("Cancelar")).clicked(){close=true;}
if ui.button(crate::i18n::tr("Descartar alterações")).clicked(){self.issue(Action::Discard(*index));
if *exit{self.issue(Action::Exit);}close=true;}
if ui.button(crate::i18n::tr("Salvar")).clicked(){self.issue(Action::SaveClose{index:*index,exit:*exit});}});},
  Dialog::Export{format,appendix,accepted}=>{ui.heading(crate::i18n::tr("Exportar paleta"));let old=*format;egui::ComboBox::from_id_salt("export-format").selected_text(crate::i18n::tr(format.label())).width(440.0).show_ui(ui,|ui|{for f in Format::ALL.into_iter().filter(|f|*f!=Format::Project){ui.selectable_value(format,f,f.label());}});
if *format!=old{*accepted=false;}ui.add_space(12.0);
if matches!(format,Format::Palette|Format::Json){ui.label(crate::i18n::tr("Preserva cores nativas, perfis, pastas, IDs, metadados, relações documentais, papéis e modos."));ui.label(crate::i18n::tr("O consumidor usa os valores resolvidos. As regras de edição ficam no projeto .colorcraft."));ui.checkbox(appendix,crate::i18n::tr("Incluir adendo informativo das relações técnicas"));}let warnings=export::warnings(&p,*format);
if !warnings.is_empty(){ui.group(|ui|{ui.label(RichText::new(crate::i18n::tr("Confira as diferenças desta saída")).color(ACCENT).strong());for warning in &warnings{ui.label(format!("• {}",crate::i18n::tr(warning)));}ui.checkbox(accepted,crate::i18n::tr("Entendi as conversões e perdas desta exportação"));});}ui.separator();ui.horizontal(|ui|{if ui.button(crate::i18n::tr("Cancelar")).clicked(){close=true;}
if ui.add_enabled(warnings.is_empty()||*accepted,egui::Button::new(crate::i18n::tr("Escolher arquivo e exportar…"))).clicked(){self.issue(Action::Export(*format,*appendix));}});},
  Dialog::Link{..}=>{if !self.link_content(ui,&mut dialog){close=true;}},
  Dialog::Composition(_)=>{},
  Dialog::Conversion{id,target,assign}=>{ui.set_min_width(640.0);ui.heading(crate::i18n::tr("Espaço de cor e perfil"));
if let Some(item)=p.item(id){let source=item.native();ui.label(crate::i18n::tr(format!("Cor: {} · origem: {}",item.label(),source.space)));egui::ComboBox::from_id_salt("conversion-target").selected_text(crate::i18n::tr(target_label(&p,target))).width(570.0).show_ui(ui,|ui|{for(space,label)in[("srgb","sRGB IEC61966-2.1"),("display-p3","Display P3"),("lab","Lab D50"),("cmyk","CMYK sintético · 300% TAC (não é uma condição de impressão medida)"),("gray","Gray Gamma 2.2")]{ui.selectable_value(target,space.into(),crate::i18n::tr(label));}for profile in &p.profiles{ui.selectable_value(target,format!("icc:{}",profile.id),format!("ICC · {}",profile.name));}});
if ui.button(crate::i18n::tr("Carregar outro perfil ICC…")).clicked(){self.issue(Action::OpenIcc);}ui.horizontal(|ui|{ui.selectable_value(assign,false,crate::i18n::tr("Converter aparência"));ui.selectable_value(assign,true,crate::i18n::tr("Atribuir perfil aos mesmos números"));});ui.label(RichText::new(crate::i18n::tr(if *assign{"Atribuir muda a interpretação dos valores sem convertê-los. Só é permitido entre modelos compatíveis."}else{"Converter recalcula os componentes para o destino: relativo colorimétrico com compensação de ponto preto."})).small().weak());
   let dest=if let Some(profile_id)=target.strip_prefix("icc:"){p.profiles.iter().find(|pr|pr.id==profile_id).ok_or_else(||"Perfil ausente".to_string()).and_then(|profile|colorcraft_core::profile_space(&profile.bytes).map(|space|NativeColor{space,components:vec![],profile_id:Some(profile_id.into())}))}else{Ok(NativeColor{space:target.clone(),components:vec![],profile_id:None})};
   let candidate=dest.and_then(|mut dest|if *assign{let compatible=dest.space==source.space||matches!(dest.space.as_str(),"srgb"|"display-p3")&&matches!(source.space.as_str(),"srgb"|"display-p3");
if !compatible{return Err("Atribuição exige o mesmo modelo de componentes.".into())}dest.components=source.components.clone();Ok(dest)}else{color::converted(&source,&dest,&p.profiles)}).and_then(|value|color::preview(&value,&p.profiles).map(|_|value));
   let mut apply_value=None;match candidate{Ok(mut value)=>{let outside=value.space!="lab"&&value.components.iter().any(|x|*x<0.0||*x>1.0);let raw=value.clone();
if outside{for v in &mut value.components{*v=v.clamp(0.0,1.0)}}let new_rgb=color::preview(&value,&p.profiles).unwrap_or(item.rgb);ui.add_space(10.0);ui.horizontal(|ui|{swatch(ui,item.rgb,&crate::i18n::tr("Antes"));crate::widgets::arrow(ui);swatch(ui,new_rgb,&crate::i18n::tr("Depois · prévia sRGB"));});ui.label(crate::i18n::tr(format!("Origem: {:?}",source.components)));ui.label(crate::i18n::tr(format!("Destino: {:?}",value.components)));let preview_oog=!color::in_gamut(new_rgb);
if preview_oog{ui.label(RichText::new(crate::i18n::tr("O destino é preservado nativamente; sua prévia pode exceder o gamut sRGB.")).color(ACCENT));}let key=egui::Id::new(("accept-conversion-gamut",id.clone(),target.clone(),*assign));let mut accepted=ctx.data(|d|d.get_temp::<bool>(key)).unwrap_or(false);
if outside{ui.group(|ui|{ui.label(RichText::new(crate::i18n::tr(format!("A conversão gera valores fora do gamut destino: {:?}",raw.components))).color(ACCENT));
if ui.checkbox(&mut accepted,crate::i18n::tr("Confirmo limitar os canais ao espaço destino; o projeto poderá ser desfeito para recuperar a origem")).changed(){ctx.data_mut(|d|d.insert_temp(key,accepted));}});}
if !outside||accepted{apply_value=Some(value)}},Err(e)=>{ui.label(RichText::new(e).color(ACCENT));}}
   ui.separator();ui.horizontal(|ui|{if ui.button(crate::i18n::tr("Cancelar")).clicked(){close=true;}
if ui.add_enabled(apply_value.is_some()&&!item.locked&&!item.source_locked&&!item.direct_locked,egui::Button::new(crate::i18n::tr(if *assign{"Confirmar atribuição"}else{"Confirmar conversão"}))).clicked()&& let Some(value)=apply_value{self.issue(Action::Confirm(Command::SetNative{id:id.clone(),value}));}});}else{ui.label(crate::i18n::tr("A cor não está mais disponível."));
if ui.button(crate::i18n::tr("Fechar")).clicked(){close=true;}}},
  Dialog::Gamut{id,original,mapped}=>{ui.heading(crate::i18n::tr("A cor solicitada está fora de sRGB"));ui.label(crate::i18n::tr("É possível reduzir sua cromaticidade mantendo luminosidade e matiz. Confira a aproximação antes de aplicar."));ui.horizontal(|ui|{swatch(ui,*original,&crate::i18n::tr("Valor anterior"));crate::widgets::arrow(ui);swatch(ui,*mapped,&crate::i18n::tr("Cromaticidade ajustada"));});ui.label(crate::i18n::tr(format!("Novo valor: {}",color::hex(*mapped))));ui.separator();ui.horizontal(|ui|{if ui.button(crate::i18n::tr("Cancelar")).clicked(){close=true;}
if ui.button(crate::i18n::tr("Aplicar aproximação e preservar desfazer")).clicked(){self.issue(Action::Confirm(Command::SetColor{id:id.clone(),rgb:*mapped,continuous:false}));}});},
  Dialog::Documentation{roles,modes,relations}=>{ui.heading(crate::i18n::tr("Relações documentais"));ui.label(crate::i18n::tr("Esta camada descreve significado e uso. Mudar uma associação não modifica os valores das cores."));egui::ScrollArea::vertical().max_height((ctx.content_rect().height()-220.0).clamp(60.0,480.0)).show(ui,|ui|{
   crate::theme::section(ui,"file","RELAÇÕES DOCUMENTAIS");let mut remove_doc=None;for(index,relation)in relations.iter_mut().enumerate(){ui.push_id(relation.id.clone(),|ui|{ui.group(|ui|{ui.horizontal(|ui|{color_combo(ui,&p,"from",&mut relation.from,true);ui.text_edit_singleline(&mut relation.kind);color_combo(ui,&p,"to",&mut relation.to,true);
if ui.small_button(crate::i18n::tr("×")).clicked(){remove_doc=Some(index)}});ui.add(egui::TextEdit::singleline(&mut relation.note).desired_width(740.0).hint_text(crate::i18n::tr("Nota sobre significado ou uso")));});});}
if let Some(index)=remove_doc{relations.remove(index);}
if ui.button(crate::i18n::tr("+ Relação documental")).clicked(){relations.push(Documentary{id:uid(),from:p.items.first().map(|i|i.id.clone()).unwrap_or_default(),to:p.items.get(1).or_else(||p.items.first()).map(|i|i.id.clone()).unwrap_or_default(),kind:"Usada com".into(),note:String::new()});}
  });ui.separator();ui.horizontal(|ui|{if ui.button(crate::i18n::tr("Cancelar")).clicked(){close=true;}
if ui.button(crate::i18n::tr("Aplicar documentação")).clicked(){self.issue(Action::Confirm(Command::Documentation{relations:relations.clone(),roles:roles.clone(),modes:modes.clone()}));}});},
  }
  if let Some(error)=&self.error{ui.add_space(8.0);ui.label(RichText::new(crate::i18n::tr(error)).color(egui::Color32::from_rgb(243,145,116)));}
  if ui.input(|i|i.key_pressed(egui::Key::Escape)){close=true;}
  });
        if close {
            reopen = false;
            self.error = None;
        }
        if reopen {
            self.dialog = Some(dialog);
        }
    }
}
pub fn swatch(ui: &mut egui::Ui, rgb: [f64; 3], label: &str) {
    ui.vertical(|ui| {
        let (r, _) = ui.allocate_exact_size(Vec2::new(70.0, 45.0), egui::Sense::hover());
        ui.painter().rect_filled(r, 5.0, wheel::rgb(rgb));
        ui.label(RichText::new(crate::i18n::tr(label)).small());
    });
}
fn target_label(p: &Project, target: &str) -> String {
    if let Some(id) = target.strip_prefix("icc:") {
        p.profiles
            .iter()
            .find(|pr| pr.id == id)
            .map(|pr| format!("ICC · {}", pr.name))
            .unwrap_or("Perfil ausente".into())
    } else {
        match target {
            "srgb" => "sRGB IEC61966-2.1",
            "display-p3" => "Display P3",
            "lab" => "Lab D50",
            "cmyk" => "CMYK sintético · 300% TAC",
            "gray" => "Gray Gamma 2.2",
            x => x,
        }
        .into()
    }
}
pub(crate) fn open_folder(path: &std::path::Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer.exe")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new(if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        })
        .arg(path)
        .spawn()
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
