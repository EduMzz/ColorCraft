use crate::{
    app::{Action, ColorCraftApp, SourceWorkbench},
    dialogs::Dialog,
};
use colorcraft_core::{capture, color, engine::Command, *};
use eframe::egui::{self, Color32, RichText, Sense, TextureOptions, Vec2};
use image::{ImageReader, RgbaImage};
use std::{
    io::Cursor,
    path::{Path, PathBuf},
    sync::Arc,
};

const MAX_SOURCE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_RASTER_PIXELS: u64 = 100_000_000;

std::thread_local! {
    static PDFIUM_INSTANCE: std::cell::RefCell<Option<pdfium_render::prelude::Pdfium>> = const { std::cell::RefCell::new(None) };
}

#[derive(Debug)]
pub struct RenderedSource {
    pub image: RgbaImage,
    pub page_count: u32,
    pub color_warning: Option<String>,
}

pub fn read_source_bytes(source: &SourceAsset) -> Result<Vec<u8>> {
    if let Some(encoded) = &source.embedded_data {
        return base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            encoded.as_str(),
        )
        .map_err(|e| e.to_string());
    }
    let path = PathBuf::from(&source.path);
    let metadata = std::fs::metadata(&path).map_err(|e| format!("Fonte indisponível: {e}"))?;
    if metadata.len() > MAX_SOURCE_BYTES {
        return Err("A fonte excede 512 MiB. Reduza o arquivo ou use uma cópia menor.".into());
    }
    std::fs::read(path).map_err(|e| e.to_string())
}

pub fn render_source(source: &SourceAsset, page: u32) -> Result<RenderedSource> {
    let bytes = read_source_bytes(source)?;
    let ext = Path::new(&source.path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    render_bytes(&bytes, &ext, page)
}

pub fn render_bytes(bytes: &[u8], ext: &str, page: u32) -> Result<RenderedSource> {
    match ext {
        "png" | "jpg" | "jpeg" | "webp" | "tif" | "tiff" => {
            if has_unsupported_icc(bytes, ext) {
                return Err("Este arquivo contém um perfil ICC que o ColorCraft não conseguiu converter com segurança para sRGB. A captura foi bloqueada para evitar diferença silenciosa de cor; converta uma cópia para sRGB no editor de imagem e tente novamente.".into());
            }
            let mut reader = ImageReader::new(Cursor::new(bytes))
                .with_guessed_format()
                .map_err(|e| e.to_string())?;
            let mut limits = image::Limits::default();
            limits.max_image_width = Some(30000);
            limits.max_image_height = Some(30000);
            limits.max_alloc = Some(512 * 1024 * 1024);
            reader.limits(limits);
            let img = reader
                .decode()
                .map_err(|e| format!("Não foi possível decodificar a imagem: {e}"))?
                .to_rgba8();
            if img.width() as u64 * img.height() as u64 > MAX_RASTER_PIXELS {
                return Err(
                    "A imagem excede 100 megapixels; reduza sua resolução antes de capturar."
                        .into(),
                );
            }
            Ok(RenderedSource {
                image: img,
                page_count: 1,
                color_warning: None,
            })
        }
        "svg" => {
            let text = String::from_utf8_lossy(bytes);
            if text.to_ascii_lowercase().contains("color-profile") {
                return Err("Este SVG declara um perfil de cor específico que não pôde ser convertido com segurança para sRGB. A captura foi bloqueada.".into());
            }
            render_svg(bytes)
        }
        "pdf" => {
            let mut rendered = render_pdf(bytes, page)?;
            if bytes.windows(9).any(|w| w == b"/ICCBased") {
                rendered.color_warning = Some("Este PDF declara cores com perfil ICC. Você pode capturar os valores dos pixels renderizados na prévia pelo PDFium. A conversão de cor não foi validada pelo ColorCraft e pode diferir de outros aplicativos. Se precisar de uma conversão validada, cancele e converta uma cópia para sRGB em outro aplicativo.".into());
            }
            Ok(rendered)
        }
        _ => Err("Formato não suportado. Use PNG, JPEG, WebP, TIFF, PDF ou SVG.".into()),
    }
}

fn has_unsupported_icc(bytes: &[u8], ext: &str) -> bool {
    match ext {
        "png" => bytes.windows(4).any(|w| w == b"iCCP"),
        "webp" => bytes.windows(4).any(|w| w == b"ICCP"),
        "jpg" | "jpeg" => bytes.windows(11).any(|w| w == b"ICC_PROFILE"),
        "tif" | "tiff" => bytes
            .windows(2)
            .any(|w| w == [0x87, 0x73] || w == [0x73, 0x87]),
        _ => false,
    }
}

fn render_svg(bytes: &[u8]) -> Result<RenderedSource> {
    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(bytes, &options)
        .map_err(|e| format!("SVG inválido ou não suportado: {e}"))?;
    let size = tree.size();
    let scale = (4096.0 / size.width().max(size.height()).max(1.0)).min(1.0);
    let width = (size.width() * scale).ceil().clamp(1.0, 4096.0) as u32;
    let height = (size.height() * scale).ceil().clamp(1.0, 4096.0) as u32;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or("Memória insuficiente para renderizar o SVG.")?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    let image = RgbaImage::from_raw(width, height, pixmap.take())
        .ok_or("Falha ao decodificar a prévia do SVG.")?;
    Ok(RenderedSource {
        image,
        page_count: 1,
        color_warning: None,
    })
}

fn render_pdf(bytes: &[u8], page: u32) -> Result<RenderedSource> {
    use pdfium_render::prelude::{PdfRenderConfig, Pdfium};
    PDFIUM_INSTANCE.with(|instance| {
        if instance.borrow().is_none() {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let dir = exe.parent().unwrap_or_else(|| Path::new("."));
            let library = Pdfium::pdfium_platform_library_name_at_path(dir);
            let bindings = match Pdfium::bind_to_library(library) {
                Ok(bindings) => bindings,
                Err(local_error) => Pdfium::bind_to_system_library().map_err(|system_error| {
                    format!("Não foi possível carregar PDFium ({local_error}; biblioteca do sistema: {system_error}). Reinstale o ColorCraft ou coloque pdfium.dll compatível ao lado do executável.")
                })?,
            };
            *instance.borrow_mut() = Some(Pdfium::new(bindings));
        }
        let instance = instance.borrow();
        let pdfium = instance.as_ref().ok_or("PDFium não foi inicializado.")?;
        let document = pdfium
            .load_pdf_from_byte_vec(bytes.to_vec(), None)
            .map_err(|e| format!("Não foi possível abrir o PDF: {e}"))?;
        let page_count = document.pages().len() as u32;
        if page_count == 0 || page_count > 10000 {
            return Err("O PDF não contém páginas ou excede 10.000 páginas.".into());
        }
        if page >= page_count {
            return Err("Página do PDF fora do intervalo.".into());
        }
        let page = document.pages().get(page as i32).map_err(|e| e.to_string())?;
        let render_config = if page.width().value >= page.height().value {
            PdfRenderConfig::new().set_target_width(4096)
        } else {
            PdfRenderConfig::new().set_target_height(4096)
        };
        let bitmap = page.render_with_config(&render_config).map_err(|e| e.to_string())?;
        let image = bitmap.as_image().map_err(|e| e.to_string())?.to_rgba8();
        if image.width() as u64 * image.height() as u64 > MAX_RASTER_PIXELS {
            return Err("A página PDF renderizada excede 100 megapixels.".into());
        }
        Ok(RenderedSource { image, page_count, color_warning: None })
    })
}

fn preview_png(image: &RgbaImage) -> Result<Vec<u8>> {
    let thumb = image::imageops::thumbnail(image, 256, 256);
    let mut bytes = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(thumb)
        .write_to(&mut bytes, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(bytes.into_inner())
}

impl ColorCraftApp {
    pub fn add_source_dialog(&mut self, ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Adicionar fontes de captura")
            .add_filter(
                "Imagem, PDF ou SVG",
                &["png", "jpg", "jpeg", "webp", "tif", "tiff", "pdf", "svg"],
            )
            .pick_file()
        else {
            return;
        };
        match create_source(&path) {
            Ok((source, rendered)) => {
                let mut workbench = SourceWorkbench::new(ctx, source, rendered.image);
                workbench.color_warning = rendered.color_warning;
                self.capture = Some(workbench);
            }
            Err(e) => self.error = Some(e),
        }
    }
    fn open_capture_loaded(
        &mut self,
        ctx: &egui::Context,
        source_id: String,
        rendered: RenderedSource,
    ) {
        let Some(source) = self
            .project()
            .and_then(|p| p.sources.iter().find(|s| s.id == source_id))
            .cloned()
        else {
            return;
        };
        let mut workbench = SourceWorkbench::new(ctx, source, rendered.image);
        workbench.color_warning = rendered.color_warning;
        self.capture = Some(workbench);
    }
    pub fn open_source_capture(&mut self, ctx: &egui::Context, source_id: &str) {
        let Some(source) = self
            .project()
            .and_then(|p| p.sources.iter().find(|s| s.id == source_id))
            .cloned()
        else {
            return;
        };
        let initial_page = source
            .captures
            .first()
            .map(|capture| capture.page)
            .unwrap_or(0);
        match render_source(&source, initial_page) {
            Ok(decoded) => {
                if let Ok(bytes) = read_source_bytes(&source) {
                    let hash = colorcraft_storage::hash(&bytes);
                    if hash != source.fingerprint {
                        self.dialog = Some(Dialog::UpdateSource {
                            id: source.id.clone(),
                            path: source.path.clone(),
                        });
                        return;
                    }
                }
                self.open_capture_loaded(ctx, source_id.to_string(), decoded);
            }
            Err(e) => self.error = Some(e),
        }
    }
    pub fn embed_source(&mut self, source_id: &str) {
        let Some(mut source) = self
            .project()
            .and_then(|p| p.sources.iter().find(|s| s.id == source_id))
            .cloned()
        else {
            return;
        };
        let bytes = match read_source_bytes(&source) {
            Ok(b) => b,
            Err(e) => {
                self.error = Some(e);
                return;
            }
        };
        if bytes.len() > 64 * 1024 * 1024 {
            self.error = Some(
                "A incorporação está limitada a 64 MiB por fonte; mantenha este arquivo vinculado."
                    .into(),
            );
            return;
        }
        let total: usize = self
            .project()
            .into_iter()
            .flat_map(|p| p.sources.iter())
            .filter_map(|s| s.embedded_data.as_ref())
            .map(|d| {
                base64::Engine::decode(&base64::engine::general_purpose::STANDARD, d.as_str())
                    .map(|b| b.len())
                    .unwrap_or(0)
            })
            .sum();
        if total + bytes.len() > 180 * 1024 * 1024 {
            self.error = Some(
                "Fontes incorporadas excederiam 180 MiB e o limite do projeto ColorCraft.".into(),
            );
            return;
        }
        source.storage = "embedded".into();
        source.embedded_data = Some(Arc::new(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            bytes,
        )));
        self.issue(Action::Core(Command::ApplySource {
            source,
            colors_to_add: vec![],
            color_values: Default::default(),
        }));
    }
    pub fn request_source_update(&mut self, source_id: &str) {
        let Some(source) = self
            .project()
            .and_then(|p| p.sources.iter().find(|s| s.id == source_id))
            .cloned()
        else {
            return;
        };
        let path = PathBuf::from(&source.path);
        if path.is_file() {
            self.dialog = Some(Dialog::UpdateSource {
                id: source.id,
                path: source.path,
            });
        } else {
            self.request_source_relink(source_id);
        }
    }
    pub fn request_source_relink(&mut self, source_id: &str) {
        let Some(id) = self.project().and_then(|p| {
            p.sources
                .iter()
                .find(|s| s.id == source_id)
                .map(|s| s.id.clone())
        }) else {
            return;
        };
        let Some(path) = rfd::FileDialog::new()
            .set_title("Relincar fonte")
            .add_filter(
                "Imagem, PDF ou SVG",
                &["png", "jpg", "jpeg", "webp", "tif", "tiff", "pdf", "svg"],
            )
            .pick_file()
        else {
            return;
        };
        self.dialog = Some(Dialog::UpdateSource {
            id,
            path: path.to_string_lossy().into_owned(),
        });
    }
    pub(crate) fn queue_update_source(&mut self, source_id: &str, path: &str) {
        self.queue_update_source_internal(source_id, path, false);
    }
    pub(crate) fn queue_update_source_with_preview(&mut self, source_id: &str, path: &str) {
        self.queue_update_source_internal(source_id, path, true);
    }
    fn queue_update_source_internal(&mut self, source_id: &str, path: &str, accept_preview: bool) {
        let Some(mut source) = self
            .project()
            .and_then(|p| p.sources.iter().find(|s| s.id == source_id))
            .cloned()
        else {
            return;
        };
        let path = PathBuf::from(path);
        let bytes = match std::fs::read(&path) {
            Ok(v) if v.len() as u64 <= MAX_SOURCE_BYTES => v,
            Ok(_) => {
                self.error = Some("A fonte excede 512 MiB.".into());
                return;
            }
            Err(e) => {
                self.error = Some(format!("Não foi possível ler a fonte: {e}"));
                return;
            }
        };
        let ext = path
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let first = match render_bytes(&bytes, &ext, 0) {
            Ok(v) => v,
            Err(e) => {
                self.error = Some(e);
                return;
            }
        };
        if let Some(warning) = &first.color_warning
            && !accept_preview
        {
            self.dialog = Some(Dialog::SourceColorUpdate {
                id: source.id.clone(),
                path: path.to_string_lossy().into_owned(),
                warning: warning.clone(),
            });
            return;
        }
        let mut values = std::collections::BTreeMap::new();
        for capture in &mut source.captures {
            let rendered = if capture.page == 0 {
                first.image.clone()
            } else {
                match render_bytes(&bytes, &ext, capture.page) {
                    Ok(v) => v.image,
                    Err(e) => {
                        self.error =
                            Some(format!("Não foi possível recalcular {}: {e}", capture.name));
                        return;
                    }
                }
            };
            if let Ok(value) = capture::sample(
                rendered.width(),
                rendered.height(),
                rendered.as_raw(),
                capture,
            ) {
                capture.value = value;
            } else {
                self.error = Some(format!(
                    "A captura {} não encontra pixels visíveis na nova fonte.",
                    capture.name
                ));
                return;
            }
            if let Some(id) = capture.color_id.as_ref() {
                values.insert(id.clone(), capture.value);
            }
        }
        source.path = path.to_string_lossy().into_owned();
        source.name = path
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or(&source.name)
            .to_string();
        source.fingerprint = colorcraft_storage::hash(&bytes);
        source.page_count = first.page_count;
        source.preview_png = match preview_png(&first.image) {
            Ok(v) => v,
            Err(e) => {
                self.error = Some(e);
                return;
            }
        };
        source.storage = "linked".into();
        source.embedded_data = None;
        self.issue(Action::Confirm(Command::ApplySource {
            source,
            colors_to_add: vec![],
            color_values: values,
        }));
    }
    pub fn remove_source(&mut self, id: &str) {
        let colors = self
            .project()
            .and_then(|p| p.sources.iter().find(|s| s.id == id))
            .map(|s| s.captures.iter().filter(|c| c.color_id.is_some()).count())
            .unwrap_or(0);
        self.dialog = Some(Dialog::RemoveSource {
            id: id.into(),
            linked_colors: colors,
        });
    }
    pub(crate) fn source_workbench(&mut self, ctx: &egui::Context) {
        let Some(mut workbench) = self.capture.take() else {
            return;
        };
        let mut apply = false;
        let mut cancel = false;
        let screen = ctx.content_rect();
        let viewport_id =
            egui::ViewportId::from_hash_of(("source-capture-workbench", &workbench.source.id));
        let capture_icon =
            image::load_from_memory(include_bytes!("../../../assets/palette-file-small.png"))
                .ok()
                .map(|image| image.into_rgba8())
                .map(|image| eframe::egui::IconData {
                    width: image.width(),
                    height: image.height(),
                    rgba: image.into_raw(),
                });
        let mut viewport = egui::ViewportBuilder::default()
            .with_title(format!(
                "{} — {}",
                crate::i18n::tr("Capturar cores da fonte"),
                workbench.source.name
            ))
            .with_inner_size(Vec2::new(
                screen.width().mul_add(0.72, 0.0).clamp(760.0, 1100.0),
                screen.height().mul_add(0.72, 0.0).clamp(520.0, 760.0),
            ))
            .with_min_inner_size([680.0, 460.0])
            .with_resizable(true)
            .with_decorations(true);
        if let Some(icon) = capture_icon {
            viewport = viewport.with_icon(icon);
        }
        ctx.show_viewport_immediate(viewport_id, viewport, |capture_ctx, _| {
            let escape = capture_ctx
                .input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
            let dismiss_confirmation = escape && workbench.pending_delete.is_some();
            if dismiss_confirmation {
                workbench.pending_delete = None;
            }
            if capture_ctx.input(|input| input.viewport().close_requested())
                || (escape && !dismiss_confirmation)
            {
                cancel = true;
                return;
            }
            let outcome = crate::capture_ui::show(capture_ctx, &mut workbench);
            apply = outcome.apply;
            cancel |= outcome.cancel;
        });
        if apply || cancel {
            ctx.send_viewport_cmd_to(viewport_id, egui::ViewportCommand::Close);
        }
        if workbench.reload_page {
            match render_source(&workbench.source, workbench.page) {
                Ok(decoded) => {
                    if workbench.color_warning != decoded.color_warning {
                        workbench.color_warning_accepted = false;
                        workbench.color_warning = decoded.color_warning;
                    }
                    workbench.set_image(decoded.image);
                    workbench.reload_page = false;
                }
                Err(e) => {
                    self.error = Some(e);
                    workbench.reload_page = false;
                }
            }
        }
        if apply {
            self.apply_workbench(workbench);
            return;
        }
        if cancel {
            return;
        }
        self.capture = Some(workbench);
    }
    fn apply_workbench(&mut self, mut wb: SourceWorkbench) {
        if !wb.color_ready() {
            self.capture = Some(wb);
            return;
        }
        if self.project().is_none() {
            return;
        }
        let bytes = match read_source_bytes(&wb.source) {
            Ok(bytes) => bytes,
            Err(error) => {
                self.error = Some(error);
                self.capture = Some(wb);
                return;
            }
        };
        if colorcraft_storage::hash(&bytes) != wb.source.fingerprint {
            self.error = Some("O arquivo de origem mudou durante a captura. Atualize a fonte explicitamente antes de aplicar.".into());
            self.capture = Some(wb);
            return;
        }
        let source_snapshot = wb.source.clone();
        let validation: Result<()> = (|| {
            let mut pages = std::collections::BTreeMap::new();
            pages.insert(wb.page, wb.image.clone());
            for capture in &mut wb.source.captures {
                if let std::collections::btree_map::Entry::Vacant(entry) = pages.entry(capture.page)
                {
                    let rendered =
                        render_source(&source_snapshot, capture.page).map_err(|error| {
                            format!(
                                "Não foi possível validar a página da captura {}: {error}",
                                capture.name
                            )
                        })?;
                    entry.insert(rendered.image);
                }
                let image = &pages[&capture.page];
                capture.value =
                    capture::sample(image.width(), image.height(), image.as_raw(), capture)
                        .map_err(|error| format!("Captura {} inválida: {error}", capture.name))?;
            }
            Ok(())
        })();
        if let Err(error) = validation {
            self.error = Some(error);
            self.capture = Some(wb);
            return;
        }
        let mut additions = Vec::new();
        let mut values = std::collections::BTreeMap::new();
        for capture in &mut wb.source.captures {
            if let Some(id) = capture.color_id.as_ref() {
                values.insert(id.clone(), capture.value);
            } else {
                let mut item = Item::new("color", None);
                item.id = uid();
                item.name = if capture.name.trim().is_empty() {
                    "Captura".into()
                } else {
                    capture.name.clone()
                };
                item.css_name = color::css_name(&item.name);
                item.rgb = capture.value;
                item.color = Some(NativeColor::srgb(capture.value));
                item.source_locked = true;
                capture.color_id = Some(item.id.clone());
                additions.push(item);
            }
        }
        self.issue(Action::Core(Command::ApplySource {
            source: wb.source,
            colors_to_add: additions,
            color_values: values,
        }));
    }
    pub fn source_panel(&mut self, ui: &mut egui::Ui) {
        let sources = self
            .project()
            .map(|p| p.sources.clone())
            .unwrap_or_default();
        egui::Frame::new().inner_margin(12.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(crate::i18n::tr("Fontes"));
                if crate::icons::text_button(ui, "plus", "Adicionar fonte").clicked() {
                    self.add_source_dialog(ui.ctx());
                }
            });
            ui.label(
                RichText::new(crate::i18n::tr(
                    "Imagens, PDF e SVG · cores capturadas são adicionadas à raiz da paleta.",
                ))
                .small()
                .weak(),
            );
            ui.separator();
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if sources.is_empty() {
                        ui.label(crate::i18n::tr(
                            "Adicione um arquivo para começar a capturar cores.",
                        ));
                    }
                    for source in sources {
                        let card_width = ui.available_width();
                        egui::Frame::group(ui.style()).show(ui, |ui| {
                            ui.set_width((card_width - 18.0).max(1.0));
                            ui.horizontal_top(|ui| {
                                let tex = ui.ctx().data(|d| {
                                    d.get_temp::<egui::TextureHandle>(egui::Id::new((
                                        "source-thumb",
                                        &source.id,
                                    )))
                                });
                                if let Some(tex) = tex {
                                    ui.image((tex.id(), Vec2::splat(42.0)));
                                } else if !source.preview_png.is_empty()
                                    && let Ok(img) = image::load_from_memory(&source.preview_png)
                                {
                                    let img = img.to_rgba8();
                                    let ci = egui::ColorImage::from_rgba_unmultiplied(
                                        [img.width() as usize, img.height() as usize],
                                        img.as_raw(),
                                    );
                                    let tex = ui.ctx().load_texture(
                                        format!("source-{}", source.id),
                                        ci,
                                        TextureOptions::LINEAR,
                                    );
                                    ui.ctx().data_mut(|d| {
                                        d.insert_temp(
                                            egui::Id::new(("source-thumb", &source.id)),
                                            tex.clone(),
                                        )
                                    });
                                    ui.image((tex.id(), Vec2::splat(42.0)));
                                }
                                ui.vertical(|ui| {
                                    ui.set_width(ui.available_width());
                                    ui.horizontal(|ui| {
                                        let name_width = (ui.available_width() - 4.0).max(24.0);
                                        let title = ui
                                            .add_sized(
                                                [name_width, 22.0],
                                                egui::Label::new(&source.name)
                                                    .truncate()
                                                    .sense(Sense::click()),
                                            )
                                            .on_hover_text(format!(
                                                "{}\n{}",
                                                source.path,
                                                crate::i18n::tr(
                                                    "Duplo clique para abrir a captura"
                                                )
                                            ));
                                        if title.double_clicked() {
                                            self.open_source_capture(ui.ctx(), &source.id);
                                        }
                                        title.context_menu(|ui| {
                                            if ui.button(crate::i18n::tr("Abrir captura")).clicked()
                                            {
                                                self.open_source_capture(ui.ctx(), &source.id);
                                                ui.close();
                                            }
                                        });
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(crate::i18n::tr(
                                                if source.storage == "embedded" {
                                                    "Incorporada"
                                                } else {
                                                    "Vinculada"
                                                },
                                            ))
                                            .small()
                                            .weak(),
                                        );
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                let more = crate::icons::button(
                                                    ui,
                                                    "more",
                                                    "Opções da fonte",
                                                );
                                                egui::Popup::menu(&more).show(|ui| {
                                                    if ui
                                                        .button(crate::i18n::tr("Abrir captura"))
                                                        .clicked()
                                                    {
                                                        self.open_source_capture(
                                                            ui.ctx(),
                                                            &source.id,
                                                        );
                                                        ui.close();
                                                    }
                                                    if ui
                                                        .button(crate::i18n::tr("Renomear fonte…"))
                                                        .clicked()
                                                    {
                                                        self.dialog = Some(Dialog::RenameSource {
                                                            id: source.id.clone(),
                                                            name: source.name.clone(),
                                                        });
                                                        ui.close();
                                                    }
                                                    if source.storage == "linked"
                                                        && ui
                                                            .button(crate::i18n::tr(
                                                                "Incorporar ao projeto",
                                                            ))
                                                            .clicked()
                                                    {
                                                        self.embed_source(&source.id);
                                                        ui.close();
                                                    }
                                                    if ui
                                                        .button(crate::i18n::tr(
                                                            "Atualizar capturas…",
                                                        ))
                                                        .clicked()
                                                    {
                                                        self.request_source_update(&source.id);
                                                        ui.close();
                                                    }
                                                    if ui
                                                        .button(crate::i18n::tr("Relincar…"))
                                                        .clicked()
                                                    {
                                                        self.request_source_relink(&source.id);
                                                        ui.close();
                                                    }
                                                    if ui
                                                        .button(crate::i18n::tr("Remover fonte…"))
                                                        .clicked()
                                                    {
                                                        self.remove_source(&source.id);
                                                        ui.close();
                                                    }
                                                });
                                            },
                                        );
                                    });
                                });
                            });
                            if !source.captures.is_empty() {
                                ui.vertical(|ui| {
                                    for c in &source.captures {
                                        ui.horizontal(|ui| {
                                            let (r, _) = ui.allocate_exact_size(
                                                Vec2::splat(16.0),
                                                Sense::hover(),
                                            );
                                            ui.painter().circle_filled(
                                                r.center(),
                                                7.0,
                                                Color32::from_rgb(
                                                    c.value[0] as u8,
                                                    c.value[1] as u8,
                                                    c.value[2] as u8,
                                                ),
                                            );
                                            let label = ui.add_sized(
                                                [ui.available_width(), 18.0],
                                                egui::Label::new(RichText::new(&c.name).small())
                                                    .truncate(),
                                            );
                                            label.on_hover_text(format!(
                                                "{} · {}",
                                                c.name,
                                                color::hex(c.value)
                                            ));
                                        });
                                    }
                                });
                            }
                        });
                    }
                });
        });
    }
}

pub(crate) fn create_source(path: &Path) -> Result<(SourceAsset, RenderedSource)> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_SOURCE_BYTES {
        return Err("A fonte excede 512 MiB.".into());
    }
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let decoded = render_bytes(&bytes, &ext, 0)?;
    let thumb = preview_png(&decoded.image)?;
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("Fonte")
        .to_string();
    Ok((
        SourceAsset {
            id: uid(),
            name: name.clone(),
            path: path.to_string_lossy().into_owned(),
            storage: "linked".into(),
            fingerprint: colorcraft_storage::hash(&bytes),
            embedded_data: None,
            preview_png: thumb,
            page_count: decoded.page_count,
            captures: vec![],
        },
        decoded,
    ))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn supported_raster_and_svg_sources_render() {
        let image = image::DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            2,
            2,
            image::Rgba([40, 90, 170, 255]),
        ));
        for (format, ext) in [
            (image::ImageFormat::Png, "png"),
            (image::ImageFormat::Jpeg, "jpg"),
            (image::ImageFormat::WebP, "webp"),
            (image::ImageFormat::Tiff, "tiff"),
        ] {
            let mut bytes = Cursor::new(Vec::new());
            image.write_to(&mut bytes, format).unwrap();
            let rendered = render_bytes(&bytes.into_inner(), ext, 0).unwrap();
            assert_eq!((rendered.image.width(), rendered.image.height()), (2, 2));
        }
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="2" height="2"><rect width="2" height="2" fill="#285aaa"/></svg>"##;
        let rendered = render_bytes(svg, "svg", 0).unwrap();
        assert_eq!((rendered.image.width(), rendered.image.height()), (2, 2));
    }

    #[test]
    fn unsupported_icc_profiles_are_blocked_before_sampling() {
        assert!(
            render_bytes(b"marker iCCP profile", "png", 0)
                .unwrap_err()
                .contains("perfil ICC")
        );
        assert!(
            render_bytes(b"marker ICC_PROFILE\0profile", "jpg", 0)
                .unwrap_err()
                .contains("perfil ICC")
        );
    }

    fn two_page_pdf() -> Vec<u8> {
        let objects = [
            "<< /Type /Catalog /Pages 2 0 R >>",
            "<< /Type /Pages /Kids [3 0 R 5 0 R] /Count 2 >>",
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << >> /Contents 4 0 R >>",
            "<< /Length 0 >>\nstream\n\nendstream",
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 100] /Resources << >> /Contents 6 0 R >>",
            "<< /Length 0 >>\nstream\n\nendstream",
        ];
        let mut pdf = b"%PDF-1.4\n".to_vec();
        let mut offsets = vec![0usize];
        for (index, object) in objects.iter().enumerate() {
            offsets.push(pdf.len());
            pdf.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", index + 1, object).as_bytes());
        }
        let xref = pdf.len();
        pdf.extend_from_slice(
            format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len()).as_bytes(),
        );
        for offset in offsets.iter().skip(1) {
            pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        pdf.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
                offsets.len()
            )
            .as_bytes(),
        );
        pdf
    }

    #[test]
    fn pdf_pages_render_independently() {
        let pdf = two_page_pdf();
        let first = render_bytes(&pdf, "pdf", 0).unwrap();
        let second = render_bytes(&pdf, "pdf", 1).unwrap();
        assert_eq!(first.page_count, 2);
        assert_eq!(second.page_count, 2);
        assert_eq!(first.image.width(), 4096);
        assert_eq!(second.image.width(), 4096);
        icc_pdf_renders_preview_with_an_explicit_choice_instead_of_rejection();
    }

    // PDFium bindings initialize once per process and the renderer belongs to
    // its thread. Exercise both PDF fixtures in the same test thread.
    fn icc_pdf_renders_preview_with_an_explicit_choice_instead_of_rejection() {
        let profile = color::profile(&NativeColor::srgb([0.0; 3]), &[])
            .unwrap()
            .to_bytes();
        let content = b"/CS1 cs 0.2 0.4 0.6 scn 0 0 100 100 re f";
        let mut stream = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
        stream.extend_from_slice(content);
        stream.extend_from_slice(b"\nendstream");
        let mut icc = format!(
            "<< /N 3 /Alternate /DeviceRGB /Length {} >>\nstream\n",
            profile.len()
        )
        .into_bytes();
        icc.extend_from_slice(&profile);
        icc.extend_from_slice(b"\nendstream");
        let objects = [
            b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
            b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
            b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << /ColorSpace << /CS1 [/ICCBased 5 0 R] >> >> /Contents 4 0 R >>".to_vec(),
            stream, icc,
        ];
        let mut pdf = b"%PDF-1.4\n".to_vec();
        let mut offsets = vec![0usize];
        for (index, object) in objects.iter().enumerate() {
            offsets.push(pdf.len());
            pdf.extend_from_slice(format!("{} 0 obj\n", index + 1).as_bytes());
            pdf.extend_from_slice(object);
            pdf.extend_from_slice(b"\nendobj\n");
        }
        let xref = pdf.len();
        pdf.extend_from_slice(
            format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len()).as_bytes(),
        );
        for offset in offsets.iter().skip(1) {
            pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        pdf.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
                offsets.len()
            )
            .as_bytes(),
        );
        let rendered = render_bytes(&pdf, "pdf", 0).unwrap();
        assert!(rendered.color_warning.is_some());
        assert_eq!(rendered.page_count, 1);
        let pixel = rendered.image.get_pixel(2048, 2048);
        assert!(pixel[0] < pixel[1] && pixel[1] < pixel[2]);
        let ctx = egui::Context::default();
        let mut wb = SourceWorkbench::new(
            &ctx,
            SourceAsset::default(),
            image::imageops::thumbnail(&rendered.image, 100, 100),
        );
        wb.color_warning = rendered.color_warning;
        assert!(!wb.color_ready());
        wb.color_warning_accepted = true;
        assert!(wb.color_ready());
    }
}
