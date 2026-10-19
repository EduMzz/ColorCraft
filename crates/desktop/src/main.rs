#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod bridge;
mod capture_mask;
mod capture_ui;
mod composition_ui;
mod conflict_log;
mod conflict_ui;
mod dialogs;
mod docking;
mod i18n;
mod icons;
mod link_ui;
mod panels;
mod preferences_ui;
mod presentation;
mod semantic_ui;
mod sources;
mod theme;
mod wheel;
mod widgets;
use std::path::PathBuf;
#[derive(Default)]
pub struct Args {
    pub bridge_receiver: Option<colorcraft_bridge::native::Receiver>,
    pub bridge_dir: Option<PathBuf>,
    pub library_dir: Option<PathBuf>,
    pub bridge_start: bool,
    pub uris: Vec<String>,
    pub config: Option<PathBuf>,
    pub data: Option<PathBuf>,
    pub files: Vec<PathBuf>,
    pub demo: bool,
    pub screenshot: Option<PathBuf>,
    pub isolated: bool,
    pub qa_scenario: Option<String>,
    pub qa_theme: Option<String>,
    pub qa_language: Option<String>,
    pub qa_scale: Option<f32>,
    pub qa_size: Option<[f32; 2]>,
}
fn main() {
    let mut args = Args {
        library_dir: std::env::var_os("COLORCRAFT_LIBRARY_DIR").map(PathBuf::from),
        ..Args::default()
    };
    let mut iter = std::env::args().skip(1);
    while let Some(a) = iter.next() {
        match a.as_str() {
            "--config-dir" => args.config = iter.next().map(PathBuf::from),
            "--data-dir" => args.data = iter.next().map(PathBuf::from),
            "--bridge-dir" => args.bridge_dir = iter.next().map(PathBuf::from),
            "--library-dir" => args.library_dir = iter.next().map(PathBuf::from),
            "--bridge-start" => args.bridge_start = true,
            _ if a.starts_with("colorcraft://") => args.uris.push(a),
            "--demo" => args.demo = true,
            "--isolated" => args.isolated = true,
            "--screenshot" => args.screenshot = iter.next().map(PathBuf::from),
            "--qa-scenario" => args.qa_scenario = iter.next(),
            "--qa-language" => args.qa_language = iter.next(),
            "--qa-theme" => args.qa_theme = iter.next(),
            "--qa-scale" => args.qa_scale = iter.next().and_then(|s| s.parse().ok()),
            "--qa-size" => {
                args.qa_size = iter.next().and_then(|s| {
                    let (w, h) = s.split_once('x')?;
                    Some([w.parse().ok()?, h.parse().ok()?])
                })
            }
            "--version" => {
                println!("ColorCraft {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            _ if !a.starts_with('-') => args.files.push(PathBuf::from(a)),
            _ => {
                eprintln!("Opção desconhecida: {a}");
                return;
            }
        }
    }
    if args.screenshot.is_some() {
        args.isolated = true;
        if args.data.is_none() {
            args.data = Some(
                std::env::temp_dir().join(format!("colorcraft-capture-{}", colorcraft_core::uid())),
            );
        }
    }
    if !args.isolated || args.bridge_dir.is_some() {
        match args
            .bridge_dir
            .clone()
            .map(Ok)
            .unwrap_or_else(colorcraft_bridge::native::root)
        {
            Ok(root) => {
                for uri in &args.uris {
                    let result = colorcraft_bridge::Request::from_uri(uri)
                        .and_then(|r| colorcraft_bridge::native::submit(&root, &r));
                    if let Err(e) = result {
                        eprintln!("ColorCraft URI: {e}");
                        return;
                    }
                }
                args.bridge_receiver =
                    colorcraft_bridge::native::Receiver::acquire(root.clone()).ok();
                if args.bridge_receiver.is_some()
                    && !args.isolated
                    && let Ok(executable) = std::env::current_exe()
                    && let Err(e) = colorcraft_bridge::native::atomic(
                        &root.join("app.json"),
                        &serde_json::json!({"schemaVersion":1,"executable":executable,"version":env!("CARGO_PKG_VERSION")}),
                    )
                {
                    eprintln!("ColorCraft registration: {e}");
                }
                if args.bridge_receiver.is_none() && (!args.uris.is_empty() || args.bridge_start) {
                    return;
                }
            }
            Err(e) => {
                eprintln!("ColorCraft bridge: {e}");
                if !args.uris.is_empty() {
                    return;
                }
            }
        }
    }
    let icon = image::load_from_memory(include_bytes!("../../../assets/colorcraft-128.png"))
        .ok()
        .map(|img| {
            let img = img.into_rgba8();
            let (w, h) = img.dimensions();
            eframe::egui::IconData {
                rgba: img.into_raw(),
                width: w,
                height: h,
            }
        });
    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_inner_size(args.qa_size.unwrap_or([1440.0, 920.0]))
        .with_min_inner_size([720.0, 480.0])
        .with_resizable(true)
        .with_visible(!args.bridge_start)
        .with_title("ColorCraft")
        .with_decorations(false);
    if let Some(icon) = icon {
        viewport = viewport.with_icon(icon)
    }
    let native = eframe::NativeOptions {
        viewport,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    if let Err(error) = eframe::run_native(
        "ColorCraft",
        native,
        Box::new(move |cc| Ok(Box::new(app::ColorCraftApp::new(cc, args)))),
    ) {
        eprintln!("Não foi possível abrir ColorCraft: {error}");
    }
}
