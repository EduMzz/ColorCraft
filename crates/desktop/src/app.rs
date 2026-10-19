use crate::{
    Args,
    dialogs::Dialog,
    docking::{self, DockArea, DockStyle, Layout, Node, PanelLimits, Permissions},
    wheel,
};
use colorcraft_core::{
    engine::{Command, CompositionDraft, Engine},
    export::Format,
    *,
};
use colorcraft_storage::{self as storage, Paths, Prefs};
use craft_ui::layout::{SplitAxis, SplitSize};
use eframe::egui::{self, Color32, Id, RichText, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
pub const ACCENT: Color32 = Color32::from_rgb(242, 190, 69);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Panel {
    Palette,
    Color,
    Links,
    Details,
    Sources,
    Roles,
    Conflicts,
}
impl Panel {
    pub const ALL: [Self; 7] = [
        Self::Palette,
        Self::Color,
        Self::Links,
        Self::Details,
        Self::Sources,
        Self::Roles,
        Self::Conflicts,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Conflicts => {
                if crate::i18n::is_english() {
                    "Conflicts"
                } else {
                    "Conflitos"
                }
            }
            Self::Palette => {
                if crate::i18n::is_english() {
                    "Palette"
                } else {
                    "Paleta"
                }
            }
            Self::Color => {
                if crate::i18n::is_english() {
                    "Color"
                } else {
                    "Cor"
                }
            }
            Self::Links => {
                if crate::i18n::is_english() {
                    "Links"
                } else {
                    "Vínculos"
                }
            }
            Self::Details => {
                if crate::i18n::is_english() {
                    "Details"
                } else {
                    "Detalhes"
                }
            }
            Self::Roles => {
                if crate::i18n::is_english() {
                    "Semantic Roles"
                } else {
                    "Papéis semânticos"
                }
            }
            Self::Sources => {
                if crate::i18n::is_english() {
                    "Sources"
                } else {
                    "Fontes"
                }
            }
        }
    }
}
impl WorkspaceStore {
    fn sync_active(&mut self, layout: &Layout<Panel>) {
        if let Some(workspace) = self.presets.iter_mut().find(|w| w.id == self.active) {
            workspace.layout = layout.clone();
        }
    }
    fn activate(&self, id: &str) -> Option<Layout<Panel>> {
        self.presets
            .iter()
            .find(|w| w.id == id)
            .map(|w| w.layout.clone())
    }
    pub(crate) fn reset(&mut self, id: &str) -> Option<Layout<Panel>> {
        let replacement = Self::defaults(default_layout())
            .presets
            .iter()
            .find(|w| w.id == id)
            .map(|w| w.layout.clone())
            .unwrap_or_else(default_layout);
        let current = self.presets.iter_mut().find(|w| w.id == id)?;
        current.layout = replacement.clone();
        Some(replacement)
    }
}
pub fn default_layout() -> Layout<Panel> {
    Layout {
        root: Some(Node::Split {
            axis: SplitAxis::Horizontal,
            size: SplitSize::FixedSecond(390.0),
            first: Box::new(Node::Tabs {
                panels: vec![Panel::Palette],
                active: 0,
            }),
            second: Box::new(Node::Tabs {
                panels: vec![
                    Panel::Color,
                    Panel::Links,
                    Panel::Details,
                    Panel::Sources,
                    Panel::Roles,
                    Panel::Conflicts,
                ],
                active: 0,
            }),
        }),
        floating: vec![],
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspacePreset {
    pub id: String,
    pub name: String,
    pub layout: Layout<Panel>,
    pub builtin: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceStore {
    pub active: String,
    pub presets: Vec<WorkspacePreset>,
}
impl WorkspaceStore {
    fn defaults(current: Layout<Panel>) -> Self {
        let mut classic = default_layout();
        classic.root = Some(Node::Split {
            axis: SplitAxis::Horizontal,
            size: SplitSize::FixedFirst(310.0),
            first: Box::new(Node::Tabs {
                panels: vec![Panel::Palette, Panel::Details],
                active: 0,
            }),
            second: Box::new(Node::Tabs {
                panels: vec![Panel::Color, Panel::Links, Panel::Roles, Panel::Conflicts],
                active: 0,
            }),
        });
        let mut color = default_layout();
        color.root = Some(Node::Split {
            axis: SplitAxis::Horizontal,
            size: SplitSize::FixedSecond(510.0),
            first: Box::new(Node::Tabs {
                panels: vec![Panel::Palette],
                active: 0,
            }),
            second: Box::new(Node::Tabs {
                panels: vec![
                    Panel::Color,
                    Panel::Details,
                    Panel::Links,
                    Panel::Roles,
                    Panel::Conflicts,
                ],
                active: 0,
            }),
        });
        let mut relations = default_layout();
        relations.root = Some(Node::Split {
            axis: SplitAxis::Horizontal,
            size: SplitSize::FixedSecond(360.0),
            first: Box::new(Node::Tabs {
                panels: vec![Panel::Palette, Panel::Color],
                active: 0,
            }),
            second: Box::new(Node::Tabs {
                panels: vec![Panel::Links, Panel::Details, Panel::Roles, Panel::Conflicts],
                active: 0,
            }),
        });
        Self {
            active: "essentials".into(),
            presets: vec![
                WorkspacePreset {
                    id: "essentials".into(),
                    name: "Essentials".into(),
                    layout: current,
                    builtin: true,
                },
                WorkspacePreset {
                    id: "essentials-classic".into(),
                    name: "Essentials Classic".into(),
                    layout: classic,
                    builtin: true,
                },
                WorkspacePreset {
                    id: "color-editing".into(),
                    name: "Edição de cor".into(),
                    layout: color,
                    builtin: true,
                },
                WorkspacePreset {
                    id: "relations".into(),
                    name: "Relações".into(),
                    layout: relations,
                    builtin: true,
                },
            ],
        }
    }
    fn valid(&self) -> bool {
        !self.presets.is_empty()
            && self.presets.len() <= 64
            && self.presets.iter().all(|w| {
                !w.id.is_empty()
                    && !w.name.trim().is_empty()
                    && w.layout.validate().is_ok()
                    && w.layout.panels().iter().all(|p| Panel::ALL.contains(p))
            })
            && self.presets.iter().any(|w| w.id == self.active)
    }
}
pub struct Document {
    pub engine: Engine,
    pub path: Option<PathBuf>,
    pub disk_hash: Option<String>,
    pub saved_hash: String,
    pub publication_pending: bool,
    pub selected: Vec<String>,
    pub active: Option<String>,
    pub folder: Option<String>,
    pub search: String,
    pub expanded: BTreeSet<String>,
    pub conflicts: crate::conflict_log::ConflictLog,
    pub reveal_item: Option<String>,
    pub focused_relation: Option<String>,
    pub scroll_relation: Option<String>,
    pub(crate) dirty_cache: std::cell::RefCell<(u64, bool)>,
}
impl Document {
    pub fn visible_item_ids(&self, tree: bool) -> Vec<String> {
        let p = &self.engine.project;
        if !self.search.is_empty() {
            let q = self.search.to_lowercase();
            return p
                .items
                .iter()
                .filter(|i| {
                    i.name.to_lowercase().contains(&q)
                        || i.css_name.to_lowercase().contains(&q)
                        || i.tags.iter().any(|t| t.to_lowercase().contains(&q))
                })
                .map(|i| i.id.clone())
                .collect();
        }
        fn visit(
            p: &Project,
            parent: Option<&str>,
            expanded: &BTreeSet<String>,
            tree: bool,
            out: &mut Vec<String>,
            depth: usize,
        ) {
            if depth > 100 {
                return;
            }
            for item in p.items.iter().filter(|i| i.parent_id.as_deref() == parent) {
                out.push(item.id.clone());
                if tree && item.kind == "folder" && expanded.contains(&item.id) {
                    visit(p, Some(&item.id), expanded, tree, out, depth + 1);
                }
            }
        }
        let mut ids = vec![];
        visit(p, self.folder.as_deref(), &self.expanded, tree, &mut ids, 0);
        ids
    }
    pub fn new(
        project: Project,
        path: Option<PathBuf>,
        disk_hash: Option<String>,
        saved: bool,
    ) -> Self {
        let active = project.colors().next().map(|i| i.id.clone());
        let selected = active.clone().into_iter().collect();
        let saved_hash = if saved {
            storage::content_hash(&project)
        } else {
            String::new()
        };
        Self {
            engine: Engine::new(project),
            path,
            disk_hash,
            saved_hash,
            publication_pending: false,
            selected,
            active,
            folder: None,
            search: String::new(),
            expanded: BTreeSet::new(),
            conflicts: Default::default(),
            reveal_item: None,
            focused_relation: None,
            scroll_relation: None,
            dirty_cache: std::cell::RefCell::new((u64::MAX, false)),
        }
    }
    pub fn dirty(&self) -> bool {
        let mut cache = self.dirty_cache.borrow_mut();
        if cache.0 != self.engine.generation {
            *cache = (
                self.engine.generation,
                storage::content_hash(&self.engine.project) != self.saved_hash,
            );
        }
        cache.1
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WorkbenchTool {
    Dropper,
    Brush,
    Eraser,
    Hand,
    Zoom,
}
pub struct SourceWorkbench {
    pub source: SourceAsset,
    pub image: image::RgbaImage,
    pub texture: egui::TextureHandle,
    pub selected: usize,
    pub tool: WorkbenchTool,
    pub zoom: f32,
    pub pan: Vec2,
    pub page: u32,
    pub brush_radius: f32,
    pub dirty: bool,
    pub reload_page: bool,
    pub pending_delete: Option<usize>,
    pub color_warning: Option<String>,
    pub color_warning_accepted: bool,
    brush_stroke: Option<usize>,
    pub(crate) moving_capture: Option<usize>,
    pub(crate) eraser_last: Option<[f32; 2]>,
    pub(crate) mask_overlay: crate::capture_mask::MaskOverlay,
}
impl SourceWorkbench {
    pub(crate) fn new(ctx: &egui::Context, source: SourceAsset, image: image::RgbaImage) -> Self {
        let page = source
            .captures
            .first()
            .map(|capture| capture.page)
            .unwrap_or(0);
        let texture = ctx.load_texture(
            format!("capture-{}", source.id),
            egui::ColorImage::from_rgba_unmultiplied(
                [image.width() as usize, image.height() as usize],
                image.as_raw(),
            ),
            egui::TextureOptions::LINEAR,
        );
        Self {
            source,
            image,
            texture,
            selected: 0,
            tool: WorkbenchTool::Dropper,
            zoom: 1.0,
            pan: Vec2::ZERO,
            page,
            brush_radius: 8.0,
            dirty: false,
            reload_page: false,
            pending_delete: None,
            color_warning: None,
            color_warning_accepted: false,
            brush_stroke: None,
            moving_capture: None,
            eraser_last: None,
            mask_overlay: Default::default(),
        }
    }
    pub fn set_image(&mut self, image: image::RgbaImage) {
        self.texture.set(
            egui::ColorImage::from_rgba_unmultiplied(
                [image.width() as usize, image.height() as usize],
                image.as_raw(),
            ),
            egui::TextureOptions::LINEAR,
        );
        self.image = image;
    }
    pub fn new_capture(&mut self, pos: [f32; 2]) {
        let mut capture = ColorCapture {
            name: format!("Captura {}", self.source.captures.len() + 1),
            page: self.page,
            position: pos,
            ..Default::default()
        };
        if let Ok(value) = colorcraft_core::capture::sample(
            self.image.width(),
            self.image.height(),
            self.image.as_raw(),
            &capture,
        ) {
            capture.value = value;
        }
        self.source.captures.push(capture);
        self.selected = self.source.captures.len() - 1;
        self.dirty = true;
    }
    pub fn start_brush(&mut self, pos: [f32; 2]) {
        // A tool change never converts the selected point capture into a mask.
        // Continue painting only a mask explicitly selected on this page.
        if !self
            .source
            .captures
            .get(self.selected)
            .is_some_and(|capture| {
                capture.page == self.page && matches!(capture.method, CaptureMethod::Brush { .. })
            })
        {
            self.new_capture(pos);
        }
        if let Some(capture) = self.source.captures.get_mut(self.selected) {
            if !matches!(capture.method, CaptureMethod::Brush { .. }) {
                capture.method = CaptureMethod::Brush {
                    statistic: "mean".into(),
                };
            }
            if capture.strokes.is_empty() {
                capture.position = pos;
            }
            capture.page = self.page;
            capture.strokes.push(CaptureStroke {
                points: vec![pos],
                radius: self.brush_radius,
            });
            self.brush_stroke = Some(capture.strokes.len() - 1);
            self.dirty = true;
        }
    }
    pub fn continue_brush(&mut self, pos: [f32; 2]) {
        if let Some(index) = self.brush_stroke
            && let Some(capture) = self.source.captures.get_mut(self.selected)
            && let Some(stroke) = capture.strokes.get_mut(index)
        {
            stroke.points.push(pos);
            self.dirty = true;
        }
    }
    pub fn end_brush(&mut self) {
        if self.brush_stroke.take().is_some() {
            self.sample_selected();
        }
    }
    pub fn sample_selected(&mut self) {
        if let Some(capture) = self.source.captures.get_mut(self.selected)
            && capture.page == self.page
            && let Ok(value) = colorcraft_core::capture::sample(
                self.image.width(),
                self.image.height(),
                self.image.as_raw(),
                capture,
            )
        {
            capture.value = value;
        }
    }
    pub fn color_ready(&self) -> bool {
        self.color_warning.is_none() || self.color_warning_accepted
    }
    pub fn marker_at(&self, image: egui::Rect, pos: egui::Pos2) -> Option<usize> {
        self.source
            .captures
            .iter()
            .enumerate()
            .filter(|(_, c)| c.page == self.page)
            .filter_map(|(index, c)| {
                let marker = image.min
                    + Vec2::new(
                        c.position[0] * image.width(),
                        c.position[1] * image.height(),
                    );
                let distance = marker.distance(pos);
                (distance <= 16.0).then_some((index, distance))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| index)
    }
    pub fn select_capture(&mut self, index: usize) {
        if let Some(capture) = self.source.captures.get(index) {
            self.selected = index;
            self.reload_page = self.page != capture.page;
            self.page = capture.page;
        }
    }
    pub fn move_capture(&mut self, index: usize, pos: [f32; 2]) {
        if let Some(capture) = self.source.captures.get_mut(index) {
            if capture.page != self.page {
                return;
            }
            let delta = [pos[0] - capture.position[0], pos[1] - capture.position[1]];
            // Moving a mask marker translates its mask rather than just its badge.
            for stroke in &mut capture.strokes {
                for point in &mut stroke.points {
                    point[0] = (point[0] + delta[0]).clamp(0.0, 1.0);
                    point[1] = (point[1] + delta[1]).clamp(0.0, 1.0);
                }
            }
            capture.position = pos;
            self.selected = index;
            self.dirty = true;
        }
        if self
            .source
            .captures
            .get(index)
            .is_some_and(|c| !matches!(c.method, CaptureMethod::Brush { .. }))
        {
            self.sample_selected();
        }
    }
}
pub enum Action {
    RevealColor(String),
    RevealRelation(String),
    BeginPick(usize),
    ChooseReplacement(String),
    CancelPick,
    Core(Command),
    Preview(Command),
    Color {
        id: String,
        rgb: [f64; 3],
        continuous: bool,
    },
    Finish,
    Confirm(Command),
    New,
    Demo,
    Open,
    OpenPath(PathBuf),
    Save(bool),
    SaveClose {
        index: usize,
        exit: bool,
    },
    Undo,
    Redo,
    Copy(bool),
    Paste,
    PasteStored,
    Select(String, bool, bool),
    Folder(Option<String>),
    Show(Dialog),
    Export(Format, bool),
    Close(usize),
    Discard(usize),
    Exit,
    ResetLayout,
    ShowPanel(Panel),
    TogglePanel(Panel),
    OpenIcc,
    RefreshLibrary,
}
pub struct ColorCraftApp {
    pub bridge: Option<colorcraft_bridge::native::Receiver>,
    pub title_edit: Option<crate::widgets::TitleEdit>,
    pub composition_ui: crate::composition_ui::CompositionUi,
    pub picking: Option<crate::composition_ui::Picking>,
    pub capture: Option<SourceWorkbench>,
    pub docs: Vec<Document>,
    pub current: usize,
    pub home: bool,
    pub settings: craft_settings::SettingsStore<Prefs>,
    pub paths: Option<Paths>,
    pub layout: Layout<Panel>,
    pub workspaces: WorkspaceStore,
    pub queue: Vec<Action>,
    pub dialog: Option<Dialog>,
    pub error: Option<String>,
    pub status: String,
    pub logo: Option<egui::TextureHandle>,
    pub wheel: egui::TextureHandle,
    pub hex_edit: String,
    pub hex_for: String,
    pub component_mode: String,
    pub library_tab: bool,
    pub library: Vec<storage::LibraryEntry>,
    pub recoveries: Vec<PathBuf>,
    pub clipboard: Option<(Project, Vec<String>, bool)>,
    pub pending_copy: Option<String>,
    pub request_paste: bool,
    pub last_autosave: Instant,
    pub last_library: Instant,
    pub layout_dirty: bool,
    pub screenshot: Option<PathBuf>,
    pub frames: usize,
    pub allow_exit: bool,
}
impl ColorCraftApp {
    pub fn new(cc: &eframe::CreationContext<'_>, args: Args) -> Self {
        Self::with_context(&cc.egui_ctx, args)
    }
    pub fn with_context(ctx: &egui::Context, args: Args) -> Self {
        let (mut settings, mut status) = storage::open_settings(args.config, args.isolated);
        if args.isolated {
            settings.update(|p| {
                p.general.autosave = false;
                p.general.auto_save_projects = false;
            });
        }
        if let Some(theme) = &args.qa_theme {
            settings.update(|p| p.appearance.theme = theme.clone());
        }
        if let Some(language) = &args.qa_language {
            settings.update(|p| p.appearance.language = language.clone());
        }
        if let Some(scale) = args.qa_scale {
            settings.update(|p| p.appearance.scale = scale);
        }
        let data_override = if args.isolated && args.data.is_none() {
            Some(std::env::temp_dir().join(format!("colorcraft-isolated-{}", uid())))
        } else {
            args.data
        };
        let paths = match Paths::resolve(&settings, data_override) {
            Ok(mut p) => {
                if let Some(library) = args.library_dir.clone() {
                    p.library = library;
                }
                if let Err(e) = p.prepare() {
                    status = format!(
                        "Biblioteca/recuperação indisponível: {e}. Ainda é possível salvar projetos em outro destino."
                    );
                }
                Some(p)
            }
            Err(e) => {
                status = format!("Armazenamento indisponível: {e}");
                None
            }
        };
        let legacy_layout = paths
            .as_ref()
            .and_then(|p| p.config.as_ref())
            .and_then(|d| storage::bounded_read(&d.join("workspace.json"), 128 * 1024).ok())
            .and_then(|b| serde_json::from_slice::<Layout<Panel>>(&b).ok())
            .filter(|l| l.validate().is_ok() && l.panels().iter().all(|p| Panel::ALL.contains(p)))
            .unwrap_or_else(default_layout);
        let workspaces = paths
            .as_ref()
            .and_then(|p| p.config.as_ref())
            .and_then(|d| storage::bounded_read(&d.join("workspaces.json"), 1024 * 1024).ok())
            .and_then(|b| serde_json::from_slice::<WorkspaceStore>(&b).ok())
            .filter(WorkspaceStore::valid)
            .unwrap_or_else(|| WorkspaceStore::defaults(legacy_layout.clone()));
        let layout = workspaces
            .presets
            .iter()
            .find(|w| w.id == workspaces.active)
            .map(|w| w.layout.clone())
            .unwrap_or(legacy_layout);
        let image = image::load_from_memory(include_bytes!("../../../assets/colorcraft.png"))
            .ok()
            .map(|im| {
                let im = im.into_rgba8();
                let size = [im.width() as usize, im.height() as usize];
                ctx.load_texture(
                    "colorcraft-logo",
                    egui::ColorImage::from_rgba_unmultiplied(size, &im),
                    egui::TextureOptions::LINEAR,
                )
            });
        let wheel = wheel::texture(ctx);
        let mut app = Self {
            bridge: args.bridge_receiver,
            title_edit: None,
            composition_ui: Default::default(),
            picking: None,
            capture: None,
            docs: vec![],
            current: 0,
            home: true,
            settings,
            paths,
            layout,
            workspaces,
            queue: vec![],
            dialog: None,
            error: None,
            status,
            logo: image,
            wheel,
            hex_edit: String::new(),
            hex_for: String::new(),
            component_mode: "HSV".into(),
            library_tab: false,
            library: vec![],
            recoveries: vec![],
            clipboard: None,
            pending_copy: None,
            request_paste: false,
            last_autosave: Instant::now(),
            last_library: Instant::now(),
            layout_dirty: false,
            screenshot: args.screenshot,
            frames: 0,
            allow_exit: false,
        };
        app.apply_theme(ctx);
        app.refresh_library();
        if args.demo {
            app.add_doc(Project::demo(), None, None, false)
        }
        for file in args.files {
            app.open_path(file)
        }
        if let Some(scenario) = args.qa_scenario {
            let p = app.project().cloned().unwrap_or_default();
            match scenario.as_str() {
                "conflicts" | "conflicts-locked" => {
                    app.add_doc(crate::conflict_log::demo_project(), None, None, true);
                    app.issue(Action::Color {
                        id: "a".into(),
                        rgb: colorcraft_core::color::from_hsv([165.0, 0.2, 0.6]),
                        continuous: false,
                    });
                    app.issue(Action::Finish);
                    app.process_actions();
                    if scenario == "conflicts-locked" {
                        if let Some(d) = app.doc_mut()
                            && let Some(i) = d.engine.project.item_mut("c")
                        {
                            i.locked = true;
                        }
                        app.issue(Action::Color {
                            id: "a".into(),
                            rgb: colorcraft_core::color::from_hsv([165.0, 0.6, 0.6]),
                            continuous: false,
                        });
                        app.issue(Action::Finish);
                        app.process_actions();
                    }
                    app.issue(Action::ShowPanel(Panel::Conflicts));
                    app.process_actions();
                }
                "capture" | "capture-delete" | "capture-eraser" => {
                    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("../../assets/palette-file-small.png");
                    if let Ok((source, rendered)) = crate::sources::create_source(&path) {
                        let mut wb = SourceWorkbench::new(ctx, source, rendered.image);
                        wb.new_capture([0.3, 0.4]);
                        wb.start_brush([0.6, 0.6]);
                        wb.continue_brush([0.65, 0.7]);
                        wb.continue_brush([0.78, 0.6]);
                        wb.continue_brush([0.6, 0.6]);
                        wb.end_brush();
                        wb.start_brush([0.38, 0.5]);
                        wb.continue_brush([0.45, 0.65]);
                        wb.end_brush();
                        wb.tool = if scenario == "capture-eraser" {
                            WorkbenchTool::Eraser
                        } else {
                            WorkbenchTool::Brush
                        };
                        if scenario == "capture-delete" {
                            wb.pending_delete = Some(1);
                        }
                        app.capture = Some(wb);
                    }
                }
                "home" => app.home = true,
                "tree" => {
                    app.settings.update(|p| p.appearance.tree_view = true);
                }
                "panels" | "panels-tree" | "floating" => {
                    app.settings
                        .update(|p| p.appearance.tree_view = scenario == "panels-tree");
                    app.layout = Layout {
                        root: Some(Node::Split {
                            axis: SplitAxis::Horizontal,
                            size: SplitSize::FixedSecond(650.0),
                            first: Box::new(Node::Tabs {
                                panels: vec![Panel::Palette],
                                active: 0,
                            }),
                            second: Box::new(Node::Split {
                                axis: SplitAxis::Horizontal,
                                size: SplitSize::FixedSecond(300.0),
                                first: Box::new(Node::Split {
                                    axis: SplitAxis::Vertical,
                                    size: SplitSize::Ratio(0.5),
                                    first: Box::new(Node::Tabs {
                                        panels: vec![Panel::Details],
                                        active: 0,
                                    }),
                                    second: Box::new(Node::Tabs {
                                        panels: vec![Panel::Color],
                                        active: 0,
                                    }),
                                }),
                                second: Box::new(Node::Tabs {
                                    panels: vec![Panel::Links],
                                    active: 0,
                                }),
                            }),
                        }),
                        floating: vec![],
                    };
                    if let Some(doc) = app.doc_mut() {
                        let ids: Vec<_> = doc
                            .engine
                            .project
                            .colors()
                            .take(3)
                            .map(|i| i.id.clone())
                            .collect();
                        let _ = doc.engine.execute(Command::LinkColors {
                            ids: ids.clone(),
                            channels: vec![3],
                        });
                        if let Some(first) =
                            ids.first().and_then(|id| doc.engine.project.item_mut(id))
                        {
                            first.direct_locked = true;
                            for n in 1..=10 {
                                first.custom.insert(
                                    format!("Campo {n}"),
                                    serde_json::json!("Metadado da cor"),
                                );
                            }
                        }
                        for n in 0..12 {
                            let mut item = Item::new("color", None);
                            item.name = format!("Amostra {}", n + 1);
                            item.rgb =
                                colorcraft_core::color::from_hsv([n as f64 * 30.0, 0.45, 0.7]);
                            doc.engine.project.items.push(item);
                        }
                    }
                    if scenario == "floating" {
                        let _ = app.layout.apply(docking::Action::Float {
                            panel: Panel::Details,
                            rect: [90.0, 220.0, 380.0, 490.0],
                        });
                    }
                }
                "multi-link" | "limit-link" => {
                    if let Some(doc) = app.doc_mut() {
                        doc.selected = doc
                            .engine
                            .project
                            .colors()
                            .take(3)
                            .map(|i| i.id.clone())
                            .collect();
                    }
                    app.open_link();
                    app.process_actions();
                    if scenario == "limit-link"
                        && let Some(Dialog::Link { kind, .. }) = &mut app.dialog
                    {
                        *kind = 1;
                    }
                }
                "workspace-manager" => {
                    let active = app.workspaces.active.clone();
                    let name = app
                        .workspaces
                        .presets
                        .iter()
                        .find(|w| w.id == active)
                        .map(|w| w.name.clone())
                        .unwrap_or_default();
                    app.dialog = Some(Dialog::Workspaces {
                        new_name: String::new(),
                        rename_name: name,
                        selected: active,
                    });
                }
                "minimum" => {
                    app.settings.update(|p| p.appearance.card_zoom = 0.085);
                }
                "composition" => {
                    app.dialog = Some(Dialog::Composition(CompositionDraft {
                        originals: p
                            .colors()
                            .next()
                            .map(|i| vec![i.id.clone()])
                            .unwrap_or_default(),
                        ..Default::default()
                    }))
                }
                "conversion" => {
                    if let Some(item) = p.colors().next() {
                        app.dialog = Some(Dialog::Conversion {
                            id: item.id.clone(),
                            target: "display-p3".into(),
                            assign: false,
                        })
                    }
                }
                "documentation" => {
                    app.dialog = Some(Dialog::Documentation {
                        roles: p.roles,
                        modes: p.modes,
                        relations: p.documentary_relations,
                    })
                }
                "preferences" | "preferences-files" => {
                    app.dialog = Some(Dialog::Preferences);
                    if scenario == "preferences-files" {
                        ctx.data_mut(|d| d.insert_temp(Id::new("preferences-page"), 1_usize));
                    }
                }
                "roles" | "modes" => {
                    if let Some(doc) = app.doc_mut() {
                        let id = doc
                            .engine
                            .project
                            .colors()
                            .next()
                            .map(|i| i.id.clone())
                            .unwrap_or_default();
                        doc.engine.project.roles.push(Role {
                            id: "qa-role".into(),
                            name: "text.primary".into(),
                            color_id: id,
                            description: "Primary reading text".into(),
                        });
                        doc.engine.project.modes.push(Mode {
                            id: "qa-dark".into(),
                            name: "Dark".into(),
                            overrides: std::collections::BTreeMap::from([(
                                "qa-role".into(),
                                String::new(),
                            )]),
                        });
                        doc.active = None;
                        doc.selected.clear();
                        doc.folder = None;
                    }
                    app.issue(Action::ShowPanel(if scenario == "roles" {
                        Panel::Roles
                    } else {
                        Panel::Details
                    }));
                }
                "tags" => {
                    if let Some(d) = app.doc_mut() {
                        let project_id = d.engine.project.id.clone();
                        if let Some(item) = d.engine.project.items.first_mut() {
                            item.tags = vec!["brand".into(), "interface".into()];
                            let key = Id::new(("item-tags", &project_id, &item.id));
                            ctx.data_mut(|data| {
                                data.insert_temp(key, "new".to_owned());
                                data.insert_temp(key.with("open"), true);
                            });
                        }
                    }
                    app.issue(Action::ShowPanel(Panel::Details));
                }
                "links" => {
                    if let Some(d) = app.doc_mut() {
                        let ids: Vec<_> = d.engine.project.colors().map(|i| i.id.clone()).collect();
                        if ids.len() >= 4 {
                            let _ = d.engine.execute(Command::Compose(CompositionDraft {
                                kind: "mono".into(),
                                originals: vec![ids[0].clone()],
                                count: 5,
                                ..Default::default()
                            }));
                            let _ = d.engine.execute(Command::LinkColors {
                                ids: ids.iter().take(3).cloned().collect(),
                                channels: vec![3, 4, 5],
                            });
                            let _ = d.engine.execute(Command::AddContrast {
                                a: ids[0].clone(),
                                b: ids[3].clone(),
                                minimum: 3.0,
                            });
                            d.active = Some(ids[0].clone());
                        }
                    }
                    app.issue(Action::ShowPanel(Panel::Links));
                }
                "details" => app.issue(Action::ShowPanel(Panel::Details)),
                _ => {}
            }
        }
        app
    }
    pub fn doc(&self) -> Option<&Document> {
        self.docs.get(self.current)
    }
    pub fn doc_mut(&mut self) -> Option<&mut Document> {
        self.docs.get_mut(self.current)
    }
    pub fn project(&self) -> Option<&Project> {
        self.doc().map(|d| &d.engine.project)
    }
    pub fn issue(&mut self, action: Action) {
        self.queue.push(action)
    }
    pub fn apply_theme(&self, ctx: &egui::Context) {
        crate::theme::apply(
            ctx,
            self.settings.get().appearance.theme != "light",
            self.settings.get().appearance.scale,
        );
    }
    pub fn add_doc(
        &mut self,
        p: Project,
        path: Option<PathBuf>,
        digest: Option<String>,
        saved: bool,
    ) {
        self.docs.push(Document::new(p, path, digest, saved));
        self.current = self.docs.len() - 1;
        self.home = false;
        self.hex_for.clear();
    }
    pub fn open_path(&mut self, path: PathBuf) {
        if let Some(index) = self
            .docs
            .iter()
            .position(|d| d.path.as_ref() == Some(&path))
        {
            self.current = index;
            self.home = false;
            return;
        }
        match storage::load(&path) {
            Ok(loaded) => {
                let imported = loaded.imported;
                let project = loaded.project;
                self.add_doc(
                    project,
                    if imported { None } else { Some(path.clone()) },
                    if imported { None } else { Some(loaded.hash) },
                    !imported,
                );
                if !imported {
                    self.remember(&path)
                }
                self.status = if imported {
                    "Importado com regras e metadados preservados. Salve como projeto ColorCraft."
                        .into()
                } else {
                    "Projeto aberto.".into()
                };
            }
            Err(e) => self.error = Some(e),
        }
    }
    fn remember(&mut self, path: &Path) {
        if let Some(p) = self.project() {
            let recent = storage::recent(p, path);
            self.settings.update(|s| {
                s.files.recent.retain(|r| r.path != recent.path);
                s.files.recent.insert(0, recent);
                s.files.recent.truncate(30);
            });
        }
    }
    fn save_current(&mut self, copy: bool) -> bool {
        if let Some(doc) = self.doc_mut() {
            doc.engine.finish_gesture();
        }
        let Some(doc) = self.doc() else { return false };
        let mut project = doc.engine.project.clone();
        let old_path = doc.path.clone();
        let old_hash = doc.disk_hash.clone();
        let path = if copy || old_path.is_none() {
            rfd::FileDialog::new()
                .set_title(if copy {
                    "Salvar cópia do projeto"
                } else {
                    "Salvar projeto ColorCraft"
                })
                .add_filter("Projeto ColorCraft", &["colorcraft"])
                .set_file_name(format!(
                    "{}.colorcraft",
                    colorcraft_core::color::css_name(&project.name)
                ))
                .save_file()
        } else {
            old_path.clone()
        };
        let Some(path) = path else { return false };
        let same = old_path.as_ref() == Some(&path);
        if copy && !same && old_path.is_some() {
            project
                .extra
                .insert("sourcePaletteId".into(), serde_json::json!(project.id));
            project.id = uid();
            project.revision = 0;
            project.created_at.clear();
        }
        let expected = if same {
            old_hash
        } else if path.exists() {
            match storage::file_hash(&path) {
                Ok(h) => Some(h),
                Err(e) => {
                    self.error = Some(e);
                    return false;
                }
            }
        } else {
            None
        };
        let Some(paths) = &self.paths else {
            self.error = Some(
                "A biblioteca local não está disponível. Confira o diretório de dados.".into(),
            );
            return false;
        };
        match storage::save(&project, &path, expected.as_deref(), paths) {
            Ok(saved) => {
                let publication = saved.publication_error;
                if let Some(doc) = self.doc_mut() {
                    doc.engine.project = saved.project;
                    doc.saved_hash = storage::content_hash(&doc.engine.project);
                    doc.publication_pending = publication.is_some();
                    doc.dirty_cache.replace((doc.engine.generation, false));
                    doc.path = Some(path.clone());
                    doc.disk_hash = Some(saved.hash);
                }
                self.remember(&path);
                self.status = "Salvo e disponível na biblioteca local.".into();
                if let Some(e) = publication {
                    self.error = Some(format!("Projeto salvo. Publicação local pendente: {e}"));
                    self.status = "Salvo · publicação pendente".into();
                }
                self.refresh_library();
                true
            }
            Err(e) => {
                self.error = Some(e);
                false
            }
        }
    }
    pub fn refresh_library(&mut self) {
        if let Some(paths) = &self.paths {
            match storage::library(paths) {
                Ok(lib) => self.library = lib.palettes,
                Err(e) => self.error = Some(e),
            }
            self.recoveries = storage::recoveries(paths);
        }
        self.last_library = Instant::now();
    }
    fn autosave_documents(&mut self) {
        let Some(paths) = self.paths.clone() else {
            return;
        };
        let mut refresh_library = false;
        for index in 0..self.docs.len() {
            if self.docs[index].dirty() {
                let doc = &mut self.docs[index];
                doc.engine.finish_gesture();
                if self.settings.get().general.autosave
                    && let Err(e) = storage::recover(&doc.engine.project, &paths)
                {
                    self.status = format!("Rascunho de recuperação pendente: {e}");
                }
                if self.settings.get().general.auto_save_projects
                    && let Some(path) = doc.path.clone()
                {
                    match storage::save(
                        &doc.engine.project,
                        &path,
                        doc.disk_hash.as_deref(),
                        &paths,
                    ) {
                        Ok(saved) => {
                            doc.engine.project = saved.project;
                            doc.saved_hash = storage::content_hash(&doc.engine.project);
                            doc.dirty_cache.replace((doc.engine.generation, false));
                            doc.disk_hash = Some(saved.hash);
                            doc.publication_pending = saved.publication_error.is_some();
                            if let Some(e) = saved.publication_error {
                                self.status = format!(
                                    "Projeto salvo automaticamente; publicação pendente: {e}"
                                );
                            } else {
                                self.status =
                                    "Projeto e paleta pública salvos automaticamente.".into();
                            }
                            refresh_library = true;
                        }
                        Err(e) => {
                            self.status = format!("Salvamento automático pendente: {e}");
                        }
                    }
                }
            } else if self.docs[index].publication_pending
                && self.settings.get().general.auto_save_projects
                && let Some(path) = self.docs[index].path.clone()
            {
                match storage::publish(&self.docs[index].engine.project, &path, &paths) {
                    Ok(()) => {
                        self.docs[index].publication_pending = false;
                        self.status = "Paleta pública publicada automaticamente.".into();
                        refresh_library = true;
                    }
                    Err(e) => {
                        self.status = format!("Publicação automática pendente: {e}");
                    }
                }
            }
        }
        if refresh_library {
            self.refresh_library();
        }
    }
    fn export_current(&mut self, format: Format, appendix: bool) {
        let Some(p) = self.project().cloned() else {
            return;
        };
        let path = rfd::FileDialog::new()
            .set_title("Exportar paleta")
            .add_filter(format.label(), &[format.extension()])
            .set_file_name(format!(
                "{}.{}",
                colorcraft_core::color::css_name(&p.name),
                format.extension()
            ))
            .save_file();
        if let Some(path) = path {
            match storage::export_file(&p, &path, format, appendix) {
                Ok(()) => {
                    self.status = format!("Exportado: {}", path.display());
                    self.dialog = None;
                }
                Err(e) => self.error = Some(e),
            }
        }
    }
    fn process_actions(&mut self) {
        let actions = std::mem::take(&mut self.queue);
        for action in actions {
            if self.picking.is_some()
                && !matches!(
                    &action,
                    Action::ChooseReplacement(_) | Action::CancelPick | Action::Folder(_)
                )
            {
                continue;
            }
            match action {
                Action::RevealColor(id) => self.reveal_conflict_color(&id),
                Action::RevealRelation(id) => self.reveal_conflict_relation(&id),
                Action::BeginPick(slot) => self.begin_composition_pick(slot),
                Action::ChooseReplacement(id) => self.choose_composition_replacement(&id),
                Action::CancelPick => self.cancel_composition_pick(),
                Action::Core(command) | Action::Confirm(command) => {
                    let added = if let Command::Add(item) = &command {
                        Some(item.id.clone())
                    } else {
                        None
                    };
                    match self.doc_mut().map(|d| d.engine.execute(command)) {
                        Some(Err(e)) => self.error = Some(e),
                        Some(Ok(())) => {
                            self.error = None;
                            self.status = "Alteração aplicada.".into();
                            if let Some(id) = added {
                                self.select(&id, false, false);
                            }
                            if self.dialog.as_ref().is_some_and(|d| d.is_command_dialog()) {
                                self.dialog = None;
                            }
                        }
                        None => {}
                    }
                    self.record_engine_conflict(false);
                    self.clean_selection();
                }
                Action::Preview(command) => {
                    if let Some(Err(e)) = self.doc_mut().map(|d| d.engine.preview(command)) {
                        self.error = Some(e)
                    }
                    self.record_engine_conflict(true);
                }
                Action::Color {
                    id,
                    rgb,
                    continuous,
                } => {
                    if let Some(doc) = self.doc_mut() {
                        match doc.engine.preview_color(&id, rgb, continuous) {
                            Ok(()) => {}
                            Err(e) => self.error = Some(e),
                        }
                    }
                    self.record_engine_conflict(true);
                }
                Action::Finish => {
                    if let Some(d) = self.doc_mut() {
                        d.conflicts.finish();
                        d.engine.finish_gesture()
                    }
                }
                Action::New => self.add_doc(Project::default(), None, None, false),
                Action::Demo => self.add_doc(Project::demo(), None, None, false),
                Action::Open => {
                    if let Some(paths) = rfd::FileDialog::new()
                        .set_title("Abrir ou importar paleta")
                        .add_filter(
                            "Paletas e projetos",
                            &[
                                "colorcraft",
                                "json",
                                "colorcraft-palette",
                                "color-palette",
                                "gpl",
                            ],
                        )
                        .pick_files()
                    {
                        for path in paths {
                            self.open_path(path)
                        }
                    }
                }
                Action::OpenPath(path) => self.open_path(path),
                Action::Save(copy) => {
                    self.save_current(copy);
                }
                Action::Undo => {
                    if let Some(d) = self.doc_mut() {
                        d.engine.undo()
                    }
                    self.clean_selection();
                }
                Action::Redo => {
                    if let Some(d) = self.doc_mut() {
                        d.engine.redo()
                    }
                    self.clean_selection();
                }
                Action::Copy(cut) => {
                    if let Some(d) = self.doc() {
                        let source = d.engine.project.clone();
                        let ids = d.selected.clone();
                        self.pending_copy=serde_json::to_string(&serde_json::json!({"format":"colorcraft-clipboard","project":source,"ids":ids,"cut":cut})).ok();
                        self.clipboard = Some((source, ids, cut));
                        self.status = if cut {
                            "Recorte preparado. Cole no mesmo projeto para mover os itens.".into()
                        } else {
                            "Cores, pastas e relações internas copiadas.".into()
                        };
                    }
                }
                Action::Paste => self.request_paste = true,
                Action::PasteStored => {
                    let parent = self.insertion_parent();
                    if let Some((source, ids, cut)) = self.clipboard.clone()
                        && let Some(doc) = self.doc_mut()
                    {
                        let same = source.id == doc.engine.project.id;
                        let cmd = if cut {
                            if !same {
                                self.error = Some(
                                    "Recortar entre projetos não é permitido; use Copiar.".into(),
                                );
                                continue;
                            }
                            Command::Move { ids, parent }
                        } else {
                            Command::Paste {
                                source,
                                ids,
                                parent,
                            }
                        };
                        match doc.engine.execute(cmd) {
                            Ok(()) => {
                                if cut {
                                    self.clipboard = None;
                                }
                            }
                            Err(e) => self.error = Some(e),
                        }
                    }
                }
                Action::Select(id, add, range) => {
                    if let Some(d) = self.doc_mut() {
                        d.focused_relation = None;
                    }
                    self.select(&id, add, range)
                }
                Action::Folder(folder) => {
                    if let Some(d) = self.doc_mut() {
                        d.folder = folder;
                        d.selected.clear();
                        d.active = None;
                    }
                }
                Action::Show(dialog) => {
                    if matches!(&dialog, Dialog::Composition(_)) {
                        self.composition_ui = Default::default();
                    }
                    if let Some(d) = self.doc_mut() {
                        d.engine.finish_gesture()
                    }
                    self.dialog = Some(dialog)
                }
                Action::Export(format, appendix) => self.export_current(format, appendix),
                Action::Close(index) => {
                    if self.docs.get(index).is_some_and(Document::dirty) {
                        self.dialog = Some(Dialog::Close { index, exit: false })
                    } else {
                        self.remove_doc(index)
                    }
                }
                Action::Discard(index) => self.remove_doc(index),
                Action::Exit => {
                    if let Some(index) = self.docs.iter().position(Document::dirty) {
                        self.current = index;
                        self.dialog = Some(Dialog::Close { index, exit: true });
                    } else {
                        self.allow_exit = true
                    }
                }
                Action::SaveClose { index, exit } => {
                    self.current = index;
                    if self.save_current(false) {
                        self.remove_doc(index);
                        if exit {
                            self.issue(Action::Exit)
                        }
                    }
                }
                Action::ResetLayout => {
                    let active = self.workspaces.active.clone();
                    self.layout = self
                        .workspaces
                        .reset(&active)
                        .unwrap_or_else(default_layout);
                    self.layout_dirty = true
                }
                Action::ShowPanel(panel) => {
                    if !self.layout.panels().contains(&&panel) {
                        let action = docking::Action::Open {
                            panel,
                            anchor: self.panel_anchor(),
                        };
                        if let Err(e) = self.layout.apply(action) {
                            self.error = Some(e.to_string())
                        }
                    }
                    if let Err(e) = self.layout.apply(docking::Action::Activate { panel }) {
                        self.error = Some(e.to_string())
                    }
                    self.layout_dirty = true;
                }
                Action::TogglePanel(panel) => {
                    let operation = if self.layout.panels().contains(&&panel) {
                        docking::Action::Close { panel }
                    } else {
                        docking::Action::Open {
                            panel,
                            anchor: self.panel_anchor(),
                        }
                    };
                    if let Err(e) = self.layout.apply(operation) {
                        self.error = Some(e.to_string())
                    } else {
                        self.layout_dirty = true;
                    }
                }
                Action::OpenIcc => {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Perfil ICC", &["icc", "icm"])
                        .pick_file()
                    {
                        match storage::import_profile(&path) {
                            Ok(profile) => {
                                if let Some(d) = self.doc_mut()
                                    && let Err(e) = d.engine.execute(Command::AddProfile(profile))
                                {
                                    self.error = Some(e)
                                }
                            }
                            Err(e) => self.error = Some(e),
                        }
                    }
                }
                Action::RefreshLibrary => self.refresh_library(),
            }
        }
    }
    fn panel_anchor(&self) -> Option<Panel> {
        [
            Panel::Color,
            Panel::Details,
            Panel::Links,
            Panel::Sources,
            Panel::Roles,
            Panel::Palette,
        ]
        .into_iter()
        .find(|panel| self.layout.panels().contains(&panel))
    }
    pub(crate) fn activate_workspace(&mut self, id: &str) {
        if let Some(layout) = self.workspaces.activate(id)
            && layout.validate().is_ok()
        {
            self.workspaces.sync_active(&self.layout);
            self.workspaces.active = id.to_owned();
            self.layout = layout;
            self.layout_dirty = true;
        }
    }
    pub(crate) fn create_workspace(&mut self, name: &str) {
        let clean = name.trim();
        if clean.is_empty() || self.workspaces.presets.len() >= 64 {
            self.error = Some("Informe um nome para o workspace (limite de 64 espaços).".into());
            return;
        }
        self.workspaces.sync_active(&self.layout);
        let base = clean
            .to_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect::<String>();
        let mut id = format!("custom-{}", base.trim_matches('-'));
        let stem = id.clone();
        let mut suffix = 2;
        while self.workspaces.presets.iter().any(|w| w.id == id) {
            id = format!("{stem}-{suffix}");
            suffix += 1;
        }
        self.workspaces.presets.push(WorkspacePreset {
            id: id.clone(),
            name: clean.to_owned(),
            layout: self.layout.clone(),
            builtin: false,
        });
        self.workspaces.active = id;
        self.layout_dirty = true;
    }
    pub(crate) fn rename_workspace(&mut self, id: &str, name: &str) {
        let name = name.trim();
        if name.is_empty() {
            return;
        }
        if let Some(workspace) = self
            .workspaces
            .presets
            .iter_mut()
            .find(|w| w.id == id && !w.builtin)
        {
            workspace.name = name.to_owned();
            self.layout_dirty = true;
        }
    }
    pub(crate) fn delete_workspace(&mut self, id: &str) {
        if self
            .workspaces
            .presets
            .iter()
            .find(|w| w.id == id)
            .is_some_and(|w| !w.builtin)
        {
            self.workspaces.presets.retain(|w| w.id != id);
            if self.workspaces.active == id {
                self.activate_workspace("essentials");
            }
            self.layout_dirty = true;
        }
    }
    fn remove_doc(&mut self, index: usize) {
        if index < self.docs.len() {
            let d = self.docs.remove(index);
            if let Some(paths) = &self.paths {
                let _ = storage::discard_recovery(&d.engine.project.id, paths);
            }
            self.current = self.current.min(self.docs.len().saturating_sub(1));
            if self.docs.is_empty() {
                self.home = true;
            }
            self.dialog = None;
        }
    }
    fn clean_selection(&mut self) {
        if let Some(d) = self.doc_mut() {
            d.selected.retain(|id| d.engine.project.item(id).is_some());
            if d.active
                .as_ref()
                .is_some_and(|id| d.engine.project.item(id).is_none())
            {
                d.active = d.selected.last().cloned()
            }
            if d.folder
                .as_ref()
                .is_some_and(|id| d.engine.project.item(id).is_none())
            {
                d.folder = None;
            }
        }
    }
    fn select(&mut self, id: &str, add: bool, range: bool) {
        let visible = self
            .doc()
            .map(|d| d.visible_item_ids(self.settings.get().appearance.tree_view))
            .unwrap_or_default();
        if let Some(d) = self.doc_mut() {
            if range {
                if let (Some(a), Some(b)) = (
                    d.active
                        .as_ref()
                        .and_then(|id| visible.iter().position(|x| x == id)),
                    visible.iter().position(|x| x == id),
                ) {
                    d.selected = visible[a.min(b)..=a.max(b)].to_vec();
                } else {
                    d.selected = vec![id.into()];
                }
            } else if add {
                if d.selected.iter().any(|x| x == id) {
                    d.selected.retain(|x| x != id)
                } else {
                    d.selected.push(id.into())
                }
            } else {
                d.selected = vec![id.into()]
            }
            d.active = Some(id.into());
        }
    }
    pub fn selected_colors(&self) -> Vec<String> {
        self.doc()
            .map(|d| {
                d.selected
                    .iter()
                    .filter(|id| d.engine.project.item(id).is_some_and(Item::is_color))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
    pub fn open_composition(&mut self) {
        let ids = self.selected_colors();
        self.issue(Action::Show(Dialog::Composition(CompositionDraft {
            kind: if ids.len() == 2 {
                "gradient".into()
            } else {
                "analogous".into()
            },
            originals: ids,
            ..CompositionDraft::default()
        })));
    }
    pub fn open_link(&mut self) {
        let colors = self.selected_colors();
        let a = colors
            .first()
            .cloned()
            .or_else(|| {
                self.project()
                    .and_then(|p| p.colors().next())
                    .map(|i| i.id.clone())
            })
            .unwrap_or_default();
        let b = colors
            .get(1)
            .cloned()
            .or_else(|| {
                self.project()
                    .and_then(|p| p.colors().find(|i| i.id != a))
                    .map(|i| i.id.clone())
            })
            .unwrap_or_default();
        self.issue(Action::Show(Dialog::Link {
            participants: if colors.len() >= 2 {
                colors
            } else {
                [a.clone(), b.clone()]
                    .into_iter()
                    .filter(|id| !id.is_empty())
                    .collect()
            },
            a,
            b,
            channels: [true, true, true, false, false, false],
            kind: 0,
            minimum: 4.5,
            parametric_type: "channels".into(),
            limit_type: "contrast".into(),
            convert: false,
            relation_kind: "Usada com".into(),
            note: String::new(),
        }));
    }
    /// Destination follows selection only in tree view. Ties use palette item order.
    pub(crate) fn insertion_parent(&self) -> Option<String> {
        let doc = self.doc()?;
        if !self.settings.get().appearance.tree_view {
            return doc.folder.clone();
        }
        let p = &doc.engine.project;
        let mut selected = doc.selected.iter().filter_map(|id| p.item(id));
        if doc.selected.len() == 1
            && let Some(item) = selected.next()
        {
            return if item.kind == "folder" {
                Some(item.id.clone())
            } else {
                item.parent_id.clone()
            };
        }
        let depth = |item: &Item| {
            let mut depth = 0;
            let mut parent = item.parent_id.as_deref();
            while let Some(id) = parent {
                depth += 1;
                if depth > 100 {
                    break;
                }
                parent = p.item(id).and_then(|i| i.parent_id.as_deref());
            }
            depth
        };
        p.items
            .iter()
            .filter(|item| doc.selected.contains(&item.id))
            .min_by_key(|item| depth(item))
            .map(|item| item.parent_id.clone())
            .unwrap_or_else(|| doc.folder.clone())
    }
    fn paste_text(&mut self, text: &str) {
        if text.len() > 32 * 1024 * 1024 {
            self.error = Some("Conteúdo da área de transferência maior que 32 MiB.".into());
            return;
        }
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(text)
            && value.get("format").and_then(|v| v.as_str()) == Some("colorcraft-clipboard")
        {
            let parsed = serde_json::from_value::<Project>(
                value.get("project").cloned().unwrap_or_default(),
            );
            match parsed {
                Ok(mut source) => {
                    if let Err(e) = source.validate() {
                        self.error = Some(e);
                        return;
                    }
                    source.freeze_harmonies();
                    let ids: Vec<String> =
                        match serde_json::from_value(value.get("ids").cloned().unwrap_or_default())
                        {
                            Ok(ids) => ids,
                            Err(e) => {
                                self.error = Some(e.to_string());
                                return;
                            }
                        };
                    let cut = value.get("cut").and_then(|v| v.as_bool()).unwrap_or(false)
                        && self
                            .clipboard
                            .as_ref()
                            .is_some_and(|(old, old_ids, was_cut)| {
                                *was_cut && old.id == source.id && old_ids == &ids
                            });
                    self.clipboard = Some((source, ids, cut));
                    self.issue(Action::PasteStored);
                }
                Err(e) => self.error = Some(e.to_string()),
            }
            return;
        }
        let lines: Vec<&str> = if text.split_whitespace().all(|s| s.starts_with('#')) {
            text.split_whitespace().collect()
        } else {
            text.split(['\n', ';'])
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect()
        };
        if lines.is_empty() {
            return;
        }
        let mut source = Project::default();
        for line in lines {
            match colorcraft_core::color::parse(line) {
                Ok(rgb) => source.items.push(Item {
                    rgb,
                    name: colorcraft_core::color::hex(rgb),
                    ..Item::default()
                }),
                Err(e) => {
                    self.error = Some(format!("Não foi possível colar a lista de cores: {e}"));
                    return;
                }
            }
        }
        let ids = source.items.iter().map(|i| i.id.clone()).collect();
        let parent = self.insertion_parent();
        self.issue(Action::Core(Command::Paste {
            source,
            ids,
            parent,
        }));
    }
    fn keyboard(&mut self, ctx: &egui::Context) {
        if self.picking.is_some() {
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.issue(Action::CancelPick);
            }
            return;
        }
        if self.dialog.is_some() {
            return;
        }
        let input = ctx.input(|i| i.clone());
        let command = input.modifiers.command;
        let shift = input.modifiers.shift;
        if command && input.key_pressed(egui::Key::N) {
            self.issue(Action::New)
        }
        if command && input.key_pressed(egui::Key::O) {
            self.issue(Action::Open)
        }
        if command && input.key_pressed(egui::Key::S) {
            self.issue(Action::Save(shift))
        }
        if command && input.key_pressed(egui::Key::W) {
            self.issue(Action::Close(self.current))
        }
        if input.key_pressed(egui::Key::F1) {
            self.issue(Action::Show(Dialog::Guide))
        }
        if ctx.text_edit_focused() {
            return;
        }
        if command && input.key_pressed(egui::Key::Z) {
            self.issue(if shift { Action::Redo } else { Action::Undo })
        }
        if command && input.key_pressed(egui::Key::Y) {
            self.issue(Action::Redo)
        }
        let copied = input
            .raw
            .events
            .iter()
            .any(|e| matches!(e, egui::Event::Copy));
        let cut = input
            .raw
            .events
            .iter()
            .any(|e| matches!(e, egui::Event::Cut));
        if copied || command && input.key_pressed(egui::Key::C) {
            self.issue(Action::Copy(false))
        }
        if cut || command && input.key_pressed(egui::Key::X) {
            self.issue(Action::Copy(true))
        }
        for event in &input.raw.events {
            if let egui::Event::Paste(text) = event {
                self.paste_text(text)
            }
        }
        if command
            && input.key_pressed(egui::Key::D)
            && let Some(d) = self.doc()
        {
            self.issue(Action::Core(Command::Duplicate(d.selected.clone())))
        }
        if command && input.key_pressed(egui::Key::A) {
            let visible = self
                .doc()
                .map(|d| d.visible_item_ids(self.settings.get().appearance.tree_view))
                .unwrap_or_default();
            if let Some(d) = self.doc_mut() {
                d.selected = visible;
            }
        }
        if input.key_pressed(egui::Key::Delete)
            && let Some(d) = self.doc()
            && !d.selected.is_empty()
        {
            self.issue(Action::Show(Dialog::Delete(d.selected.clone())))
        }
    }
    fn titlebar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("titlebar")
            .exact_size(crate::theme::APP_BAR)
            .frame(
                egui::Frame::new()
                    .fill(ui.visuals().extreme_bg_color)
                    .inner_margin(egui::Margin::symmetric(8, 0)),
            )
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    let compact = ui.ctx().content_rect().width() < 720.0;
                    ui.spacing_mut().item_spacing.x = if compact { 4.0 } else { 8.0 };
                    if let Some(logo) = &self.logo {
                        ui.image((logo.id(), Vec2::splat(20.0)));
                    }
                    if crate::icons::toggle(ui, "home", "Home", self.home).clicked()
                        && self.picking.is_none()
                    {
                        self.home = true;
                    }
                    ui.add_enabled_ui(self.picking.is_none(), |ui| {
                        ui.allocate_ui_with_layout(
                            Vec2::new(if compact { 220.0 } else { 246.0 }, 28.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                egui::MenuBar::new().ui(ui, |ui| self.menus(ui));
                            },
                        );
                    });
                    let gap = ui.spacing().item_spacing.x;
                    let captions = 3.0 * (crate::icons::caption_width(ui.ctx()) + gap);
                    let workspace_width =
                        (ui.available_width() - captions - 2.0 * gap).clamp(0.0, 174.0);
                    let workspace_width = if workspace_width >= 64.0 {
                        workspace_width
                    } else {
                        0.0
                    };
                    let available =
                        (ui.available_width() - captions - workspace_width - 2.0 * gap).max(0.0);
                    let (rect, response) = ui.allocate_exact_size(
                        Vec2::new(available, 28.0),
                        egui::Sense::click_and_drag(),
                    );
                    let title = self
                        .doc()
                        .filter(|_| !self.home)
                        .map(|d| {
                            format!(
                                "{}{} · ColorCraft",
                                d.engine.project.name,
                                if d.dirty() { " *" } else { "" }
                            )
                        })
                        .unwrap_or_else(|| "ColorCraft".into());
                    let title = ui.painter().layout_no_wrap(
                        title,
                        egui::FontId::proportional(12.0),
                        ui.visuals().weak_text_color(),
                    );
                    if title.size().x + 16.0 <= rect.width() {
                        ui.painter().galley(
                            rect.center() - title.size() / 2.0,
                            title,
                            ui.visuals().weak_text_color(),
                        );
                    }
                    if response.drag_started() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag)
                    }
                    if response.double_clicked() {
                        let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
                    }
                    if workspace_width > 0.0 {
                        ui.add_enabled_ui(self.picking.is_none(), |ui| {
                            self.workspace_menu(ui, workspace_width)
                        });
                    }
                    if crate::icons::caption(ui, "minimize", "Minimizar janela").clicked() {
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                    let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
                    if crate::icons::caption(
                        ui,
                        if maximized { "restore" } else { "maximize" },
                        if maximized {
                            "Restaurar janela"
                        } else {
                            "Maximizar janela"
                        },
                    )
                    .clicked()
                    {
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
                    }
                    if crate::icons::caption(ui, "close", "Fechar ColorCraft").clicked() {
                        self.issue(Action::Exit);
                    }
                });
            });
    }
    fn workspace_menu(&mut self, ui: &mut egui::Ui, width: f32) {
        let active_name = self
            .workspaces
            .presets
            .iter()
            .find(|w| w.id == self.workspaces.active)
            .map(|w| {
                let name = if w.builtin {
                    crate::i18n::tr(&w.name)
                } else {
                    w.name.clone()
                };
                name.chars().take(14).collect::<String>()
            })
            .unwrap_or_else(|| "Essentials".into());
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 22.0), egui::Sense::click());
        ui.painter().rect_filled(
            rect,
            4,
            if response.hovered() {
                ui.visuals().widgets.hovered.bg_fill
            } else {
                ui.visuals().panel_fill
            },
        );
        ui.painter()
            .with_clip_rect(egui::Rect::from_min_max(
                rect.min,
                rect.max - Vec2::new(26.0, 0.0),
            ))
            .text(
                rect.left_center() + Vec2::new(10.0, 0.0),
                egui::Align2::LEFT_CENTER,
                &active_name,
                egui::FontId::proportional(12.0),
                ui.visuals().text_color(),
            );
        crate::icons::paint(
            ui,
            egui::Rect::from_center_size(
                rect.right_center() - Vec2::new(12.0, 0.0),
                Vec2::splat(12.0),
            ),
            "down",
            ui.visuals().weak_text_color(),
        );
        let response = response.on_hover_text(crate::i18n::tr("Trocar workspace"));
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Button,
                ui.is_enabled(),
                crate::i18n::tr("Trocar workspace"),
            )
        });
        #[cfg(test)]
        ui.ctx()
            .data_mut(|d| d.insert_temp(Id::new("test-workspace-button"), rect));
        egui::Popup::menu(&response).show(|ui| {
            for workspace in self.workspaces.presets.clone() {
                let display_name = if workspace.builtin {
                    crate::i18n::tr(&workspace.name)
                } else {
                    workspace.name.clone()
                };
                if ui
                    .selectable_label(workspace.id == self.workspaces.active, display_name)
                    .clicked()
                {
                    self.activate_workspace(&workspace.id);
                    ui.close();
                }
            }
            ui.separator();
            let built_in = self
                .workspaces
                .presets
                .iter()
                .find(|w| w.id == self.workspaces.active)
                .is_some_and(|w| w.builtin);
            let current_name = self
                .workspaces
                .presets
                .iter()
                .find(|w| w.id == self.workspaces.active)
                .map(|w| w.name.clone())
                .unwrap_or_else(|| "Workspace".into());
            if built_in
                && ui
                    .button(crate::i18n::tr(format!("Redefinir {current_name}")))
                    .clicked()
            {
                self.issue(Action::ResetLayout);
                ui.close();
            }
            if ui
                .button(crate::i18n::tr("Novo espaço de trabalho…"))
                .clicked()
            {
                self.dialog = Some(Dialog::Workspaces {
                    new_name: String::new(),
                    rename_name: current_name.clone(),
                    selected: self.workspaces.active.clone(),
                });
                ui.close();
            }
            if ui
                .button(crate::i18n::tr("Gerenciar espaços de trabalho…"))
                .clicked()
            {
                self.dialog = Some(Dialog::Workspaces {
                    new_name: String::new(),
                    rename_name: current_name,
                    selected: self.workspaces.active.clone(),
                });
                ui.close();
            }
        });
    }

    fn menus(&mut self, ui: &mut egui::Ui) {
        let has = self.doc().is_some();
        let any = self.doc().is_some_and(|d| !d.selected.is_empty());
        ui.menu_button(crate::i18n::tr("Arquivo"), |ui| {
            for (label, action) in [
                ("Nova paleta     Ctrl+N", Action::New),
                ("Abrir / importar…     Ctrl+O", Action::Open),
            ] {
                if ui.button(crate::i18n::tr(label)).clicked() {
                    self.issue(action);
                    ui.close();
                }
            }
            ui.separator();
            if ui
                .add_enabled(has, egui::Button::new(crate::i18n::tr("Salvar     Ctrl+S")))
                .clicked()
            {
                self.issue(Action::Save(false));
                ui.close();
            }
            if ui
                .add_enabled(
                    has,
                    egui::Button::new(crate::i18n::tr("Salvar cópia como…     Ctrl+Shift+S")),
                )
                .clicked()
            {
                self.issue(Action::Save(true));
                ui.close();
            }
            if ui
                .add_enabled(has, egui::Button::new(crate::i18n::tr("Exportar…")))
                .clicked()
            {
                self.issue(Action::Show(Dialog::Export {
                    format: Format::Palette,
                    appendix: false,
                    accepted: false,
                }));
                ui.close();
            }
            ui.separator();
            if ui
                .add_enabled(
                    has,
                    egui::Button::new(crate::i18n::tr("Detalhes do projeto…")),
                )
                .clicked()
            {
                if let Some(p) = self.project() {
                    self.issue(Action::Show(Dialog::Project {
                        name: p.name.clone(),
                        description: p.description.clone(),
                    }));
                }
                ui.close();
            }
            if ui
                .add_enabled(
                    has,
                    egui::Button::new(crate::i18n::tr("Fechar paleta     Ctrl+W")),
                )
                .clicked()
            {
                self.issue(Action::Close(self.current));
                ui.close();
            }
            ui.separator();
            if ui.button(crate::i18n::tr("Sair")).clicked() {
                self.issue(Action::Exit);
                ui.close();
            }
        });
        ui.menu_button(crate::i18n::tr("Editar"), |ui| {
            let undo = self
                .doc()
                .and_then(|d| d.engine.undo.last())
                .map(|h| crate::i18n::tr(format!("Desfazer {}     Ctrl+Z", h.label)))
                .unwrap_or("Desfazer     Ctrl+Z".into());
            if ui
                .add_enabled(
                    self.doc().is_some_and(|d| !d.engine.undo.is_empty()),
                    egui::Button::new(undo),
                )
                .clicked()
            {
                self.issue(Action::Undo);
                ui.close();
            }
            if ui
                .add_enabled(
                    self.doc().is_some_and(|d| !d.engine.redo.is_empty()),
                    egui::Button::new(crate::i18n::tr("Refazer     Ctrl+Shift+Z")),
                )
                .clicked()
            {
                self.issue(Action::Redo);
                ui.close();
            }
            ui.separator();
            for (label, action) in [
                ("Copiar     Ctrl+C", Action::Copy(false)),
                ("Recortar     Ctrl+X", Action::Copy(true)),
            ] {
                if ui
                    .add_enabled(any, egui::Button::new(crate::i18n::tr(label)))
                    .clicked()
                {
                    self.issue(action);
                    ui.close();
                }
            }
            if ui
                .add_enabled(has, egui::Button::new(crate::i18n::tr("Colar     Ctrl+V")))
                .clicked()
            {
                self.issue(Action::Paste);
                ui.close();
            }
            if ui
                .add_enabled(
                    any,
                    egui::Button::new(crate::i18n::tr("Duplicar     Ctrl+D")),
                )
                .clicked()
            {
                if let Some(d) = self.doc() {
                    self.issue(Action::Core(Command::Duplicate(d.selected.clone())));
                }
                ui.close();
            }
            if ui
                .add_enabled(
                    any,
                    egui::Button::new(crate::i18n::tr("Excluir…     Delete")),
                )
                .clicked()
            {
                if let Some(d) = self.doc() {
                    self.issue(Action::Show(Dialog::Delete(d.selected.clone())));
                }
                ui.close();
            }
            ui.separator();
            if ui.button(crate::i18n::tr("Preferências…")).clicked() {
                self.issue(Action::Show(Dialog::Preferences));
                ui.close();
            }
        });
        ui.menu_button(crate::i18n::tr("Paleta"), |ui| {
            if ui
                .add_enabled(has, egui::Button::new(crate::i18n::tr("Nova cor")))
                .clicked()
            {
                let parent = self.insertion_parent();
                self.issue(Action::Core(Command::Add(Item::new("color", parent))));
                ui.close();
            }
            if ui
                .add_enabled(has, egui::Button::new(crate::i18n::tr("Nova pasta")))
                .clicked()
            {
                let parent = self.insertion_parent();
                self.issue(Action::Core(Command::Add(Item::new("folder", parent))));
                ui.close();
            }
            ui.separator();
            if ui
                .add_enabled(
                    !self.selected_colors().is_empty(),
                    egui::Button::new(crate::i18n::tr("Explorar harmonias / gradiente…")),
                )
                .clicked()
            {
                self.open_composition();
                ui.close();
            }
            if ui
                .add_enabled(
                    self.project().is_some_and(|p| p.count() > 1),
                    egui::Button::new(crate::i18n::tr("Criar vínculo ou limite…")),
                )
                .clicked()
            {
                self.open_link();
                ui.close();
            }
            if ui
                .add_enabled(
                    has,
                    egui::Button::new(crate::i18n::tr("Relações documentais…")),
                )
                .clicked()
            {
                if let Some(p) = self.project() {
                    self.issue(Action::Show(Dialog::Documentation {
                        roles: p.roles.clone(),
                        modes: p.modes.clone(),
                        relations: p.documentary_relations.clone(),
                    }));
                }
                ui.close();
            }
            if ui
                .add_enabled(
                    has,
                    egui::Button::new(crate::i18n::tr("Carregar perfil ICC…")),
                )
                .clicked()
            {
                self.issue(Action::OpenIcc);
                ui.close();
            }
        });
        ui.menu_button(crate::i18n::tr("Janela"), |ui| {
            for panel in Panel::ALL {
                let visible = self.layout.panels().contains(&&panel);
                if ui.selectable_label(visible, panel.label()).clicked() {
                    self.issue(Action::TogglePanel(panel));
                    ui.close();
                }
            }
            ui.separator();
            if ui
                .button(crate::i18n::tr("Redefinir disposição deste workspace"))
                .clicked()
            {
                self.issue(Action::ResetLayout);
                ui.close();
            }
            if ui.button(crate::i18n::tr("Home")).clicked() {
                self.home = true;
                ui.close();
            }
        });
        ui.menu_button(crate::i18n::tr("Ajuda"), |ui| {
            if ui
                .button(crate::i18n::tr("Guia do ColorCraft     F1"))
                .clicked()
            {
                self.issue(Action::Show(Dialog::Guide));
                ui.close();
            }
            if ui.button(crate::i18n::tr("Licenças e créditos")).clicked() {
                self.issue(Action::Show(Dialog::Credits));
                ui.close();
            }
            ui.separator();
            if ui
                .button(crate::i18n::tr("ArtCraft · projeto de referência ↗"))
                .clicked()
            {
                if let Err(e) = webbrowser::open("https://github.com/storytold") {
                    self.error = Some(e.to_string())
                }
                ui.close();
            }
            if ui.button(crate::i18n::tr("Sobre ColorCraft")).clicked() {
                self.issue(Action::Show(Dialog::About));
                ui.close();
            }
        });
    }
    fn tabs(&mut self, ui: &mut egui::Ui) {
        if self.docs.is_empty() {
            return;
        }
        egui::Panel::top("document-tabs")
            .exact_size(crate::theme::TAB_BAR)
            .frame(egui::Frame::NONE.fill(ui.visuals().extreme_bg_color))
            .show(ui, |ui| {
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 1.0;
                        for index in 0..self.docs.len() {
                            let d = &self.docs[index];
                            let label = format!(
                                "{}{}",
                                d.engine.project.name,
                                if d.dirty() { " •" } else { "" }
                            );
                            let active = !self.home && index == self.current;
                            let width =
                                (label.chars().count() as f32 * 6.7 + 48.0).clamp(130.0, 300.0);
                            let (rect, response) = ui
                                .allocate_exact_size(Vec2::new(width, 31.0), egui::Sense::click());
                            ui.painter().rect_filled(
                                rect,
                                0,
                                if active {
                                    ui.visuals().panel_fill
                                } else {
                                    ui.visuals().extreme_bg_color
                                },
                            );
                            if active {
                                ui.painter().line_segment(
                                    [rect.left_top(), rect.right_top()],
                                    Stroke::new(2.0, ACCENT),
                                );
                            }
                            let shown = label
                                .chars()
                                .take(((width - 44.0) / 6.7) as usize)
                                .collect::<String>();
                            ui.painter().text(
                                rect.left_center() + Vec2::new(12.0, 0.0),
                                egui::Align2::LEFT_CENTER,
                                shown,
                                egui::FontId::proportional(12.0),
                                if active {
                                    ui.visuals().text_color()
                                } else {
                                    ui.visuals().weak_text_color()
                                },
                            );
                            response.widget_info(|| {
                                egui::WidgetInfo::selected(
                                    egui::WidgetType::SelectableLabel,
                                    ui.is_enabled(),
                                    active,
                                    &label,
                                )
                            });
                            let close_rect = egui::Rect::from_center_size(
                                rect.right_center() - Vec2::new(16.0, 0.0),
                                Vec2::splat(24.0),
                            );
                            let over_close = ui.input(|i| {
                                i.pointer
                                    .hover_pos()
                                    .is_some_and(|p| close_rect.contains(p))
                            });
                            if self.picking.is_none() {
                                if crate::icons::at(ui, close_rect, "close", "Fechar paleta", false)
                                    .clicked()
                                {
                                    self.issue(Action::Close(index));
                                }
                                if response.clicked() && !over_close {
                                    self.current = index;
                                    self.home = false;
                                    self.hex_for.clear();
                                }
                            }
                            response.on_hover_text(label);
                        }
                    });
                });
            });
    }
    fn home_ui(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(32.0);
                let max = ui.available_width().min(1120.0);
                ui.horizontal(|ui| {
                    ui.add_space(((ui.available_width() - max) / 2.0).max(0.0));
                    ui.vertical(|ui| {
                        ui.set_max_width(max);
                        ui.horizontal(|ui| {
                            if let Some(logo) = &self.logo {
                                ui.image((logo.id(), Vec2::splat(104.0)));
                            }
                            ui.add_space(16.0);
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(crate::i18n::tr("COLORCRAFT"))
                                        .size(12.0)
                                        .color(ACCENT)
                                        .strong(),
                                );
                                ui.heading(
                                    RichText::new(crate::i18n::tr("Seu laboratório de cores."))
                                        .size(32.0),
                                );
                                ui.label(
                                    RichText::new(crate::i18n::tr(
                                        "Crie paletas que guardam valores, relações e significado.",
                                    ))
                                    .color(ui.visuals().weak_text_color()),
                                );
                            });
                        });
                        ui.add_space(24.0);
                        ui.horizontal(|ui| {
                            if ui
                                .add_sized(
                                    [170.0, 39.0],
                                    egui::Button::new(
                                        RichText::new(crate::i18n::tr("+  Nova paleta"))
                                            .color(Color32::from_rgb(30, 27, 20))
                                            .strong(),
                                    )
                                    .fill(ACCENT)
                                    .stroke(Stroke::NONE),
                                )
                                .clicked()
                            {
                                self.issue(Action::New)
                            }
                            if ui
                                .add_sized(
                                    [150.0, 39.0],
                                    egui::Button::new(crate::i18n::tr("Abrir / importar…")),
                                )
                                .clicked()
                            {
                                self.issue(Action::Open)
                            }
                            if ui
                                .add_sized(
                                    [130.0, 39.0],
                                    egui::Button::new(crate::i18n::tr("Explorar exemplo")),
                                )
                                .clicked()
                            {
                                self.issue(Action::Demo)
                            }
                        });
                        ui.add_space(27.0);
                        ui.horizontal(|ui| {
                            ui.selectable_value(
                                &mut self.library_tab,
                                false,
                                crate::i18n::tr("Projetos recentes"),
                            );
                            ui.selectable_value(
                                &mut self.library_tab,
                                true,
                                crate::i18n::tr("Biblioteca local"),
                            );
                            if ui.small_button(crate::i18n::tr("Atualizar")).clicked() {
                                self.issue(Action::RefreshLibrary)
                            }
                        });
                        ui.separator();
                        ui.add_space(10.0);
                        if !self.recoveries.is_empty() {
                            ui.group(|ui| {
                                ui.label(
                                    RichText::new(crate::i18n::tr(
                                        "Rascunhos de recuperação disponíveis",
                                    ))
                                    .color(ACCENT),
                                );
                                for path in self.recoveries.clone() {
                                    if let Ok(loaded) = storage::load(&path) {
                                        ui.horizontal(|ui| {
                                            ui.label(&loaded.project.name);
                                            if ui
                                                .button(crate::i18n::tr("Recuperar uma cópia"))
                                                .clicked()
                                            {
                                                let mut p = loaded.project;
                                                p.extra.insert(
                                                    "recoveredFrom".into(),
                                                    serde_json::json!(p.id),
                                                );
                                                p.id = uid();
                                                p.revision = 0;
                                                self.add_doc(p, None, None, false);
                                            }
                                        });
                                    }
                                }
                            });
                            ui.add_space(12.0);
                        }
                        let entries: Vec<_> = if self.library_tab {
                            self.library
                                .iter()
                                .map(|e| {
                                    (
                                        e.name.clone(),
                                        e.colors,
                                        e.preview.clone(),
                                        PathBuf::from(&e.source_path),
                                        format!("Revisão {} · biblioteca local", e.revision),
                                    )
                                })
                                .collect()
                        } else {
                            self.settings
                                .get()
                                .files
                                .recent
                                .iter()
                                .map(|e| {
                                    (
                                        e.name.clone(),
                                        e.colors,
                                        e.preview.clone(),
                                        PathBuf::from(&e.path),
                                        e.updated_at.chars().take(10).collect::<String>(),
                                    )
                                })
                                .collect()
                        };
                        if entries.is_empty() {
                            ui.add_space(48.0);
                            ui.label(
                                RichText::new(if self.library_tab {
                                    "Suas paletas salvas aparecerão aqui automaticamente."
                                } else {
                                    "Comece com uma nova paleta ou abra um projeto do Colorator."
                                })
                                .size(17.0)
                                .color(ui.visuals().weak_text_color()),
                            );
                        }
                        let cols = ((max / 310.0).floor() as usize).max(1);
                        egui::Grid::new("home-cards")
                            .spacing([16.0, 16.0])
                            .show(ui, |ui| {
                                for (index, (name, count, preview, path, date)) in
                                    entries.iter().enumerate()
                                {
                                    let size =
                                        Vec2::new((max / cols as f32 - 18.0).max(200.0), 156.0);
                                    let (rect, response) =
                                        ui.allocate_exact_size(size, egui::Sense::click());
                                    let p = ui.painter();
                                    p.rect_filled(rect, 9.0, ui.visuals().faint_bg_color);
                                    p.rect_stroke(
                                        rect,
                                        9.0,
                                        Stroke::new(
                                            1.0,
                                            if response.hovered() {
                                                ACCENT
                                            } else {
                                                ui.visuals().widgets.noninteractive.bg_stroke.color
                                            },
                                        ),
                                        egui::StrokeKind::Inside,
                                    );
                                    let strip = rect.shrink2(Vec2::new(12.0, 12.0));
                                    let n = preview.len().max(1);
                                    for (i, color) in preview.iter().enumerate() {
                                        let r = egui::Rect::from_min_size(
                                            strip.min
                                                + Vec2::new(
                                                    i as f32 * strip.width() / n as f32,
                                                    0.0,
                                                ),
                                            Vec2::new(strip.width() / n as f32 + 0.5, 65.0),
                                        );
                                        p.rect_filled(r, 0.0, wheel::rgb(*color));
                                    }
                                    p.text(
                                        rect.min + Vec2::new(14.0, 101.0),
                                        egui::Align2::LEFT_CENTER,
                                        name,
                                        egui::FontId::proportional(15.0),
                                        ui.visuals().text_color(),
                                    );
                                    p.text(
                                        rect.min + Vec2::new(14.0, 129.0),
                                        egui::Align2::LEFT_CENTER,
                                        format!("{count} cores · {date}"),
                                        egui::FontId::proportional(11.0),
                                        ui.visuals().weak_text_color(),
                                    );
                                    if response.clicked() {
                                        self.issue(Action::OpenPath(path.clone()))
                                    }
                                    response.on_hover_text(path.to_string_lossy());
                                    if (index + 1) % cols == 0 {
                                        ui.end_row();
                                    }
                                }
                            });
                    });
                });
            });
        });
    }
    fn editor(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new())
            .show(ui, |ui| {
                let layout = self.layout.clone();
                let mut style = DockStyle::from_ui(ui);
                style.tab_height = 33.0;
                style.gap = 6.0;
                style.accent = ACCENT;
                style.active_background = ui.visuals().widgets.inactive.bg_fill;
                style.tab_background = ui.visuals().panel_fill;
                style.text = ui.visuals().text_color();
                style.inactive_text = ui.visuals().weak_text_color();
                style.font = egui::FontId::proportional(13.0);
                style.float_label = crate::i18n::tr("Desencaixar painel");
                style.close_label = crate::i18n::tr("Fechar painel");
                style.move_label = crate::i18n::tr("Mover para grupo");
                style.panels_label = crate::i18n::tr("Painéis");
                style.resize_label = crate::i18n::tr("Redimensionar painéis");
                style.resize_window_label = crate::i18n::tr("Redimensionar painel flutuante");
                let blocked = self.dialog.is_some();
                if blocked || self.picking.is_some() {
                    docking::cancel_drag::<Panel>(ui.ctx(), Id::new("colorcraft-workspace"));
                }
                let result = DockArea::new(Id::new("colorcraft-workspace")).show_with_limits(
                    ui,
                    &layout,
                    &style,
                    |p| p.label().into(),
                    |_| Permissions::default(),
                    |p| PanelLimits {
                        min: Vec2::new(
                            match p {
                                Panel::Color => 300.0,
                                Panel::Palette => 240.0,
                                _ => 160.0,
                            },
                            150.0,
                        ),
                        ..Default::default()
                    },
                    |ui, panel| {
                        if !blocked && (self.picking.is_none() || *panel == Panel::Palette) {
                            self.panel(ui, *panel)
                        } else {
                            ui.add_enabled_ui(false, |ui| self.panel(ui, *panel));
                        }
                    },
                );
                if let Some(error) = result.error {
                    self.error = Some(format!("Disposição de painéis: {error}"));
                }
                for action in result.actions {
                    if !blocked && self.picking.is_none() {
                        if let Err(e) = self.layout.apply(action) {
                            self.error = Some(e.to_string())
                        } else {
                            self.layout_dirty = true
                        }
                    }
                }
            });
    }
    fn statusbar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("status")
            .exact_size(31.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let dirty = self.doc().is_some_and(Document::dirty);
                    let (mark, _) =
                        ui.allocate_exact_size(Vec2::new(9.0, 18.0), egui::Sense::hover());
                    ui.painter().circle_filled(
                        mark.center(),
                        3.0,
                        if dirty {
                            ACCENT
                        } else {
                            Color32::from_rgb(117, 175, 144)
                        },
                    );
                    if dirty {
                        ui.label(
                            RichText::new(crate::i18n::tr("Alterações não salvas"))
                                .small()
                                .color(ACCENT),
                        );
                    }
                    let unread = self.doc().map(|d| d.conflicts.unread()).unwrap_or(0);
                    let notice = self
                        .doc()
                        .map(|d| d.engine.notice.as_str())
                        .filter(|s| !s.is_empty())
                        .unwrap_or(&self.status);
                    let notice = crate::i18n::tr(notice);
                    let width = (ui.available_width() - 240.0).max(20.0);
                    ui.add_sized(
                        Vec2::new(width, 20.0),
                        egui::Label::new(RichText::new(notice).small()).truncate(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let label = format!("{} {}", crate::i18n::tr("Conflitos"), unread);
                        let indicator = ui
                            .scope(|ui| {
                                if unread > 0 {
                                    ui.visuals_mut().override_text_color =
                                        Some(crate::theme::accent_text(ui));
                                }
                                crate::icons::text_button(ui, "conflict", &label)
                            })
                            .inner
                            .on_hover_text(format!(
                                "{}: {} · {}: {}",
                                crate::i18n::tr("Novos"),
                                unread,
                                crate::i18n::tr("No histórico"),
                                self.doc().map(|d| d.conflicts.entries.len()).unwrap_or(0)
                            ));
                        #[cfg(test)]
                        ui.ctx().data_mut(|d| {
                            d.insert_temp(egui::Id::new("test-conflicts-indicator"), indicator.rect)
                        });
                        if indicator.clicked() {
                            self.issue(Action::ShowPanel(Panel::Conflicts));
                        }
                        ui.label(
                            RichText::new(concat!("ColorCraft ", env!("CARGO_PKG_VERSION")))
                                .small()
                                .weak(),
                        );
                    });
                });
            });
    }
    fn periodic(&mut self, ctx: &egui::Context) {
        let tick = self.settings.tick(Instant::now());
        if let craft_settings::SaveStatus::Failed(e) = tick.status {
            self.error = Some(format!("Falha ao salvar preferências: {e}"));
        }
        if let Some(deadline) = tick.next_deadline {
            ctx.request_repaint_after(deadline.saturating_duration_since(Instant::now()));
        }
        if self.last_autosave.elapsed() > Duration::from_secs(15) {
            self.autosave_documents();
            if self.layout_dirty
                && let Some(path) = self.paths.as_ref().and_then(|p| p.config.as_ref())
            {
                self.workspaces.sync_active(&self.layout);
                let result = serde_json::to_vec(&self.layout)
                    .map_err(|e| e.to_string())
                    .and_then(|bytes| storage::atomic(&path.join("workspace.json"), &bytes))
                    .and_then(|()| serde_json::to_vec(&self.workspaces).map_err(|e| e.to_string()))
                    .and_then(|bytes| storage::atomic(&path.join("workspaces.json"), &bytes));
                match result {
                    Ok(()) => self.layout_dirty = false,
                    Err(e) => self.error = Some(e),
                }
            }
            self.last_autosave = Instant::now();
        }
        if self.home && self.last_library.elapsed() > Duration::from_secs(5) {
            self.refresh_library()
        }
        ctx.request_repaint_after(Duration::from_secs(2));
    }
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        crate::i18n::set_language(&self.settings.get().appearance.language);
        let ctx = ui.ctx().clone();
        self.frames += 1;
        if self.screenshot.is_some()
            && let Some(mut capture) = self.capture.take()
        {
            // The Glow backend does not deliver screenshot events for immediate child
            // viewports. Isolated visual QA hosts the identical workbench in the root.
            crate::capture_ui::show(ui, &mut capture);
            self.capture = Some(capture);
            self.screenshot_frame(&ctx);
            return;
        }
        if self.capture.is_some() {
            self.source_workbench(&ctx);
            self.process_actions();
            // Keep the parent drawn, including its chrome, while the independent
            // capture viewport owns project edits.
            ui.disable();
            self.titlebar(ui);
            self.tabs(ui);
            self.statusbar(ui);
            self.editor(ui);
            if let Some(error) = &self.error {
                egui::Window::new("Aviso de captura").show(&ctx, |ui| {
                    ui.label(crate::i18n::tr(error));
                });
            }
            self.periodic(&ctx);
            return;
        }
        self.keyboard(&ctx);
        self.titlebar(ui);
        self.tabs(ui);
        self.statusbar(ui);
        if self.picking.is_some() {
            egui::Panel::top("composition-picking-banner").frame(egui::Frame::NONE.fill(ui.visuals().selection.bg_fill).inner_margin(10)).show(ui,|ui|{
                ui.horizontal_wrapped(|ui|{
                    ui.label(crate::i18n::tr("Escolha na paleta a cor que será substituída. Você pode abrir pastas. Escape retorna à composição."));
                    if ui.button(crate::i18n::tr("Voltar à composição")).clicked(){self.issue(Action::CancelPick);}
                });
            });
        }
        if let Some(error) = self.error.clone() {
            egui::Panel::top("error")
                .frame(
                    egui::Frame::new()
                        .fill(if ui.visuals().dark_mode {
                            Color32::from_rgb(72, 37, 33)
                        } else {
                            Color32::from_rgb(255, 224, 216)
                        })
                        .inner_margin(10),
                )
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(crate::i18n::tr(error)).color(
                            if ui.visuals().dark_mode {
                                Color32::from_rgb(255, 218, 206)
                            } else {
                                Color32::from_rgb(105, 35, 24)
                            },
                        ));
                        if ui.small_button(crate::i18n::tr("Fechar aviso")).clicked() {
                            self.error = None;
                        }
                    });
                });
        }
        if self.home || self.docs.is_empty() {
            self.home_ui(ui)
        } else {
            self.editor(ui)
        }
        self.bridge_tick(&ctx);
        self.show_dialogs(&ctx);
        self.process_actions();
        if ctx.input(|i| {
            i.pointer.any_released()
                || i.key_pressed(egui::Key::Enter)
                || i.key_pressed(egui::Key::Escape)
        }) && let Some(d) = self.doc_mut()
        {
            d.conflicts.finish();
        }
        if let Some(text) = self.pending_copy.take() {
            ctx.copy_text(text)
        }
        if self.request_paste {
            self.request_paste = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaste);
        }
        if ctx.input(|i| i.pointer.any_released())
            && self.dialog.is_none()
            && let Some(d) = self.doc_mut()
        {
            d.engine.finish_gesture()
        }
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        for f in dropped
            .into_iter()
            .filter(|_| self.picking.is_none())
            .collect::<Vec<_>>()
        {
            self.open_path(f.path().to_path_buf())
        }
        if ctx.input(|i| i.viewport().close_requested()) && !self.allow_exit {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.issue(Action::Exit);
        }
        if self.allow_exit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        self.screenshot_frame(&ctx);
        self.resize_window_edges(ui);
        self.periodic(&ctx);
    }
    fn screenshot_frame(&mut self, ctx: &egui::Context) {
        if self.frames == 8 && self.screenshot.is_some() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        for event in ctx.input(|i| i.raw.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event
                && let Some(path) = self.screenshot.take()
            {
                let bytes: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
                match image::save_buffer(
                    &path,
                    &bytes,
                    image.width() as u32,
                    image.height() as u32,
                    image::ColorType::Rgba8,
                ) {
                    Ok(()) => self.allow_exit = true,
                    Err(e) => self.error = Some(e.to_string()),
                }
            }
        }
        if self.screenshot.is_some() {
            ctx.request_repaint();
        }
        if self.allow_exit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
    fn resize_window_edges(&self, ui: &mut egui::Ui) {
        let bounds = ui.max_rect();
        let edge = 7.0;
        let corner = 11.0;
        let handles = [
            (
                "west",
                egui::Rect::from_min_max(
                    bounds.min,
                    egui::pos2(bounds.left() + edge, bounds.bottom()),
                ),
                egui::ResizeDirection::West,
                egui::CursorIcon::ResizeWest,
            ),
            (
                "east",
                egui::Rect::from_min_max(
                    egui::pos2(bounds.right() - edge, bounds.top()),
                    bounds.max,
                ),
                egui::ResizeDirection::East,
                egui::CursorIcon::ResizeEast,
            ),
            (
                "north",
                egui::Rect::from_min_max(
                    bounds.min,
                    egui::pos2(bounds.right(), bounds.top() + edge),
                ),
                egui::ResizeDirection::North,
                egui::CursorIcon::ResizeNorth,
            ),
            (
                "south",
                egui::Rect::from_min_max(
                    egui::pos2(bounds.left(), bounds.bottom() - edge),
                    bounds.max,
                ),
                egui::ResizeDirection::South,
                egui::CursorIcon::ResizeSouth,
            ),
            (
                "north-west",
                egui::Rect::from_min_size(bounds.min, egui::Vec2::splat(corner)),
                egui::ResizeDirection::NorthWest,
                egui::CursorIcon::ResizeNwSe,
            ),
            (
                "north-east",
                egui::Rect::from_min_size(
                    egui::pos2(bounds.right() - corner, bounds.top()),
                    egui::Vec2::splat(corner),
                ),
                egui::ResizeDirection::NorthEast,
                egui::CursorIcon::ResizeNeSw,
            ),
            (
                "south-west",
                egui::Rect::from_min_size(
                    egui::pos2(bounds.left(), bounds.bottom() - corner),
                    egui::Vec2::splat(corner),
                ),
                egui::ResizeDirection::SouthWest,
                egui::CursorIcon::ResizeNeSw,
            ),
            (
                "south-east",
                egui::Rect::from_min_size(
                    bounds.max - egui::Vec2::splat(corner),
                    egui::Vec2::splat(corner),
                ),
                egui::ResizeDirection::SouthEast,
                egui::CursorIcon::ResizeNwSe,
            ),
        ];
        for (name, rect, direction, cursor) in handles {
            let response = ui.interact(
                rect,
                egui::Id::new(("colorcraft-window-resize", name)),
                egui::Sense::drag(),
            );
            if response.hovered() || response.dragged() {
                ui.ctx().set_cursor_icon(cursor);
            }
            if response.drag_started() {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::BeginResize(direction));
            }
        }
        let grip = egui::Rect::from_min_size(
            bounds.max - egui::Vec2::splat(12.0),
            egui::Vec2::splat(12.0),
        );
        for offset in [4.0, 7.0, 10.0] {
            ui.painter().line_segment(
                [
                    egui::pos2(grip.right() - offset, grip.bottom() - 2.0),
                    egui::pos2(grip.right() - 2.0, grip.bottom() - offset),
                ],
                egui::Stroke::new(1.0, ui.visuals().weak_text_color()),
            );
        }
    }
}
impl eframe::App for ColorCraftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.draw(ui)
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.autosave_documents();
        if self.settings.get().general.autosave
            && let Some(paths) = &self.paths
        {
            for doc in &self.docs {
                if doc.dirty()
                    && let Err(e) = storage::recover(&doc.engine.project, paths)
                {
                    eprintln!("Recuperação: {e}")
                }
            }
        }
        if let Err(e) = self.settings.flush() {
            eprintln!("Preferências: {e}")
        };
        if let Some(dir) = self.paths.as_ref().and_then(|p| p.config.as_ref())
            && let Ok(bytes) = serde_json::to_vec(&self.layout)
            && let Err(e) = storage::atomic(&dir.join("workspace.json"), &bytes)
        {
            eprintln!("Painéis: {e}")
        }
        self.workspaces.sync_active(&self.layout);
        if let Some(dir) = self.paths.as_ref().and_then(|p| p.config.as_ref())
            && let Ok(bytes) = serde_json::to_vec(&self.workspaces)
            && let Err(e) = storage::atomic(&dir.join("workspaces.json"), &bytes)
        {
            eprintln!("Workspaces: {e}")
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    fn app() -> (egui::Context, ColorCraftApp, PathBuf) {
        let ctx = egui::Context::default();
        let root = std::env::temp_dir().join(format!("colorcraft-ui-test-{}", uid()));
        let app = ColorCraftApp::with_context(
            &ctx,
            Args {
                isolated: true,
                data: Some(root.clone()),
                demo: true,
                ..Default::default()
            },
        );
        (ctx, app, root)
    }
    fn frame(
        ctx: &egui::Context,
        app: &mut ColorCraftApp,
        w: f32,
        h: f32,
        time: f64,
        events: Vec<egui::Event>,
    ) {
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(w, h))),
                time: Some(time),
                events,
                focused: true,
                ..Default::default()
            },
            |ui| app.draw(ui),
        );
        assert!(!output.shapes.is_empty());
        fn texts(shape: &egui::Shape, out: &mut Vec<String>) {
            match shape {
                egui::Shape::Text(t) => out.push(t.galley.job.text.clone()),
                egui::Shape::Vec(v) => {
                    for s in v {
                        texts(s, out)
                    }
                }
                _ => {}
            }
        }
        let mut rendered = vec![];
        for s in &output.shapes {
            texts(&s.shape, &mut rendered);
        }
        ctx.data_mut(|d| d.insert_temp(Id::new("test-rendered-text"), rendered));
        output.textures_delta.clear();
    }
    #[test]
    fn conflict_actions_log_once_per_gesture_and_never_activate_the_panel_or_dirty_the_project() {
        let (ctx, mut app, root) = app();
        app.add_doc(crate::conflict_log::demo_project(), None, None, true);
        let before = app.project().unwrap().clone();
        let layout = app.layout.clone();
        for _ in 0..12 {
            app.issue(Action::Color {
                id: "a".into(),
                rgb: colorcraft_core::color::from_hsv([165.0, 0.2, 0.6]),
                continuous: false,
            });
            app.process_actions();
        }
        let d = app.doc().unwrap();
        assert_eq!(d.conflicts.entries.len(), 1);
        assert_eq!(d.conflicts.entries[0].repetitions, 1);
        assert_eq!(d.conflicts.unread(), 1);
        assert!(!d.dirty());
        assert!(d.engine.undo.is_empty());
        assert_eq!(app.layout, layout);
        assert_eq!(app.project().unwrap(), &before);
        app.issue(Action::Finish);
        app.process_actions();
        app.issue(Action::Color {
            id: "a".into(),
            rgb: colorcraft_core::color::from_hsv([165.0, 0.1, 0.6]),
            continuous: false,
        });
        app.process_actions();
        assert_eq!(app.doc().unwrap().conflicts.entries[0].repetitions, 2);
        for (n, w) in [1440.0, 600.0].into_iter().enumerate() {
            frame(&ctx, &mut app, w, 720.0, n as f64 + 1.0, vec![]);
            let r = ctx
                .data(|d| d.get_temp::<egui::Rect>(Id::new("test-conflicts-indicator")))
                .unwrap();
            assert!(r.right() <= w, "{w}: {r:?}");
        }
        app.add_doc(Project::default(), None, None, true);
        assert!(app.doc().unwrap().conflicts.entries.is_empty());
        app.current -= 1;
        assert_eq!(app.doc().unwrap().conflicts.entries.len(), 1);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn conflict_context_menu_unlinks_preserving_colors_and_undo_restores_live_targets() {
        let (ctx, mut app, root) = app();
        app.add_doc(crate::conflict_log::demo_project(), None, None, true);
        let before = app.project().unwrap().clone();
        app.issue(Action::Color {
            id: "a".into(),
            rgb: colorcraft_core::color::from_hsv([165.0, 0.2, 0.6]),
            continuous: false,
        });
        app.process_actions();
        app.issue(Action::ShowPanel(Panel::Conflicts));
        app.process_actions();
        let mut tick = 1.0;
        for _ in 0..3 {
            frame(&ctx, &mut app, 1440.0, 920.0, tick, vec![]);
            tick += 0.2;
        }
        assert_eq!(app.doc().unwrap().conflicts.unread(), 0);
        let r = rect(&ctx, "test-conflict-relation", "saturation");
        for pressed in [true, false] {
            frame(
                &ctx,
                &mut app,
                1440.0,
                920.0,
                tick,
                vec![
                    egui::Event::PointerMoved(r.center()),
                    egui::Event::PointerButton {
                        pos: r.center(),
                        button: egui::PointerButton::Secondary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ],
            );
            tick += 0.2;
        }
        frame(&ctx, &mut app, 1440.0, 920.0, tick, vec![]);
        tick += 0.2;
        let remove = rect(&ctx, "test-conflict-remove", "saturation");
        click(
            &ctx,
            &mut app,
            remove.center(),
            Default::default(),
            &mut tick,
        );
        assert!(app.project().unwrap().links.is_empty());
        assert_eq!(app.project().unwrap().items, before.items);
        assert_eq!(app.doc().unwrap().conflicts.entries.len(), 1);
        assert!(crate::conflict_log::relation(app.project().unwrap(), "saturation").is_none());
        app.issue(Action::Undo);
        app.process_actions();
        assert_eq!(app.project().unwrap(), &before);
        assert!(crate::conflict_log::relation(app.project().unwrap(), "saturation").is_some());
        assert_eq!(app.doc().unwrap().conflicts.entries.len(), 1);
        app.issue(Action::Redo);
        app.process_actions();
        assert!(app.project().unwrap().links.is_empty());
        assert_eq!(app.project().unwrap().items, before.items);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn conflict_navigation_reveals_hidden_colors_and_overrides_relation_filter_without_edits() {
        let (ctx, mut app, root) = app();
        let mut p = crate::conflict_log::demo_project();
        let mut folder = Item::new("folder", None);
        folder.id = "folder".into();
        p.items.push(folder);
        p.item_mut("c").unwrap().parent_id = Some("folder".into());
        app.add_doc(p.clone(), None, None, true);
        app.settings.update(|p| p.appearance.tree_view = true);
        app.doc_mut().unwrap().search = "missing".into();
        app.issue(Action::RevealColor("c".into()));
        app.process_actions();
        app.process_actions();
        let d = app.doc().unwrap();
        assert_eq!(d.selected, vec!["c"]);
        assert!(d.expanded.contains("folder"));
        assert!(d.search.is_empty());
        assert!(d.folder.is_none());
        app.issue(Action::Select("unrelated".into(), false, false));
        app.issue(Action::RevealRelation("saturation".into()));
        app.process_actions();
        app.process_actions();
        for n in 0..3 {
            frame(&ctx, &mut app, 1440.0, 920.0, n as f64 + 1.0, vec![]);
        }
        let d = app.doc().unwrap();
        assert_eq!(d.active.as_deref(), Some("unrelated"));
        assert_eq!(d.focused_relation.as_deref(), Some("saturation"));
        assert!(d.scroll_relation.is_none());
        assert_eq!(app.project().unwrap(), &p);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn composition_conflicts_record_only_user_parameter_or_replacement_changes() {
        let (ctx, mut app, root) = app();
        let mut p = crate::conflict_log::demo_project();
        p.item_mut("a").unwrap().locked = true;
        app.add_doc(p.clone(), None, None, true);
        app.dialog = Some(Dialog::Composition(CompositionDraft {
            kind: "complementary".into(),
            originals: vec!["b".into()],
            ..Default::default()
        }));
        for n in 0..3 {
            frame(&ctx, &mut app, 1440.0, 920.0, n as f64 + 1.0, vec![]);
        }
        assert!(app.doc().unwrap().conflicts.entries.is_empty());
        if let Some(Dialog::Composition(draft)) = &mut app.dialog {
            draft.replacements.insert(
                1,
                colorcraft_core::engine::Replacement {
                    id: "a".into(),
                    ..Default::default()
                },
            );
            draft.replacements.insert(
                99,
                colorcraft_core::engine::Replacement {
                    id: "c".into(),
                    ..Default::default()
                },
            );
        }
        // Repairing a stale slot while rendering is not an attempted user edit.
        frame(&ctx, &mut app, 1440.0, 920.0, 3.5, vec![]);
        assert!(app.doc().unwrap().conflicts.entries.is_empty());
        app.issue(Action::BeginPick(1));
        app.process_actions();
        app.issue(Action::ChooseReplacement("a".into()));
        app.process_actions();
        frame(&ctx, &mut app, 1440.0, 920.0, 4.0, vec![]);
        assert_eq!(app.doc().unwrap().conflicts.entries.len(), 1);
        for n in 5..12 {
            frame(&ctx, &mut app, 1440.0, 920.0, n as f64, vec![]);
        }
        let d = app.doc().unwrap();
        assert_eq!(d.conflicts.entries.len(), 1);
        assert_eq!(d.conflicts.entries[0].repetitions, 1);
        assert_eq!(d.engine.project, p);
        assert!(d.engine.undo.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn relationship_editor_channels_are_named_and_controls_fit_small_windows() {
        for dark in [true, false] {
            for (w, h) in [(1440.0, 920.0), (600.0, 453.0)] {
                let (ctx, mut app, root) = app();
                crate::theme::apply(&ctx, dark, 1.0);
                app.open_link();
                app.process_actions();
                let original = app.project().unwrap().clone();
                for n in 0..3 {
                    frame(&ctx, &mut app, w, h, n as f64 + 1.0, vec![]);
                }
                let bounds = egui::Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(w, h));
                let apply = ctx
                    .data(|d| d.get_temp::<egui::Rect>(Id::new("test-link-apply")))
                    .unwrap();
                assert!(bounds.contains_rect(apply), "{w}x{h}: {apply:?}");
                for letter in ['R', 'G', 'B', 'H', 'S', 'V'] {
                    let rect = ctx
                        .data(|d| d.get_temp::<egui::Rect>(Id::new(("test-link-channel", letter))))
                        .unwrap();
                    assert!(
                        rect.width() > 90.0 && rect.height() >= 50.0,
                        "{letter}: {rect:?}"
                    );
                    assert!(rect.right() <= w);
                }
                assert_eq!(app.project().unwrap(), &original);
                if let Some(Dialog::Link { kind, .. }) = &mut app.dialog {
                    *kind = 1;
                }
                for n in 3..6 {
                    frame(&ctx, &mut app, w, h, n as f64 + 1.0, vec![]);
                }
                let apply = ctx
                    .data(|d| d.get_temp::<egui::Rect>(Id::new("test-link-apply")))
                    .unwrap();
                assert!(bounds.contains_rect(apply));
                assert_eq!(app.project().unwrap(), &original);
                std::fs::remove_dir_all(root).unwrap();
            }
        }
    }

    #[test]
    fn hidden_panels_reopen_in_populated_and_empty_workspaces_without_editing_project() {
        let (_ctx, mut app, root) = app();
        let original = app.project().unwrap().clone();
        for panel in Panel::ALL {
            app.issue(Action::ShowPanel(panel));
            app.process_actions();
            assert!(app.error.is_none(), "{:?}", app.error);
            app.issue(Action::TogglePanel(panel));
            app.process_actions();
            assert!(!app.layout.panels().contains(&&panel));
            app.issue(Action::TogglePanel(panel));
            app.process_actions();
            assert!(app.layout.panels().contains(&&panel));
            assert!(app.error.is_none(), "{:?}", app.error);
            app.layout.validate().unwrap();
        }
        for panel in Panel::ALL {
            if app.layout.panels().contains(&&panel) {
                app.issue(Action::TogglePanel(panel));
                app.process_actions();
            }
        }
        assert!(app.layout.root.is_none());
        app.issue(Action::ShowPanel(Panel::Roles));
        app.process_actions();
        assert!(app.layout.panels().contains(&&Panel::Roles));
        assert!(app.error.is_none());
        assert_eq!(app.project().unwrap(), &original);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn new_mode_popup_keeps_text_focus_and_enter_creates_an_empty_binding() {
        let (ctx, mut app, root) = app();
        let role = Role {
            id: uid(),
            name: "primary".into(),
            ..Default::default()
        };
        app.doc_mut()
            .unwrap()
            .engine
            .project
            .roles
            .push(role.clone());
        let original: Vec<_> = app.project().unwrap().colors().cloned().collect();
        app.issue(Action::ShowPanel(Panel::Roles));
        app.process_actions();
        let mut tick = 1.0;
        for _ in 0..3 {
            frame(&ctx, &mut app, 1440.0, 920.0, tick, vec![]);
            tick += 0.2;
        }
        let button = rect(&ctx, "test-add-role-mode", &role.id).center();
        click(&ctx, &mut app, button, Default::default(), &mut tick);
        frame(&ctx, &mut app, 1440.0, 920.0, tick, vec![]);
        tick += 0.2;
        let field = ctx
            .data(|d| d.get_temp::<egui::Rect>(Id::new("test-new-role-mode")))
            .unwrap();
        click(
            &ctx,
            &mut app,
            field.center(),
            Default::default(),
            &mut tick,
        );
        frame(
            &ctx,
            &mut app,
            1440.0,
            920.0,
            tick,
            vec![egui::Event::Text("night".into())],
        );
        tick += 0.2;
        assert!(app.project().unwrap().modes.is_empty());
        frame(
            &ctx,
            &mut app,
            1440.0,
            920.0,
            tick,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
        );
        let p = app.project().unwrap();
        assert_eq!(p.modes.len(), 1);
        assert_eq!(p.modes[0].name, "night");
        assert_eq!(
            p.modes[0].overrides.get(&role.id).map(String::as_str),
            Some("")
        );
        assert_eq!(p.colors().cloned().collect::<Vec<_>>(), original);
        app.doc_mut().unwrap().engine.undo();
        assert!(app.project().unwrap().modes.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn workspace_and_caption_controls_fit_compact_headers_and_center_vertically() {
        let (ctx, mut app, root) = app();
        for (index, width) in [1440.0, 900.0, 600.0, 480.0].into_iter().enumerate() {
            ctx.data_mut(|d| d.remove::<egui::Rect>(Id::new("test-workspace-button")));
            frame(&ctx, &mut app, width, 720.0, index as f64 + 1.0, vec![]);
            for name in ["minimize", "maximize", "close"] {
                let rect = ctx
                    .data(|d| d.get_temp::<egui::Rect>(Id::new(("test-caption", name))))
                    .unwrap();
                assert!(
                    rect.right() <= width && rect.left() >= 0.0,
                    "{width}: {name}: {rect:?}"
                );
            }
            if let Some(rect) =
                ctx.data(|d| d.get_temp::<egui::Rect>(Id::new("test-workspace-button")))
            {
                assert!((rect.center().y - 16.0).abs() < 1.0, "{rect:?}");
                assert!((rect.height() - 22.0).abs() < 0.1);
            }
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn tree_selection_controls_add_and_paste_destinations_without_changing_card_navigation() {
        let (_ctx, mut app, root) = app();
        let mut p = Project::default();
        for (id, kind, parent) in [
            ("folder", "folder", None),
            ("nested", "folder", Some("folder")),
            ("inside", "color", Some("nested")),
            ("sibling", "color", Some("folder")),
            ("root", "color", None),
        ] {
            let mut item = Item::new(kind, parent.map(String::from));
            item.id = id.into();
            p.items.push(item);
        }
        app.doc_mut().unwrap().engine = Engine::new(p);
        app.settings.update(|p| p.appearance.tree_view = true);
        for (selection, expected) in [
            (vec!["folder"], Some("folder")),
            (vec!["inside"], Some("nested")),
            (vec!["inside", "sibling"], Some("folder")),
            (vec!["nested", "inside"], Some("folder")),
            (vec!["inside", "root"], None),
        ] {
            app.doc_mut().unwrap().selected = selection.into_iter().map(String::from).collect();
            assert_eq!(app.insertion_parent().as_deref(), expected);
        }
        app.doc_mut().unwrap().selected = vec!["nested".into()];
        app.paste_text("#123456\n#abcdef");
        app.process_actions();
        assert!(
            app.project()
                .unwrap()
                .items
                .iter()
                .rev()
                .take(2)
                .all(|i| i.parent_id.as_deref() == Some("nested"))
        );
        let source = app.project().unwrap().clone();
        app.clipboard = Some((source, vec!["root".into()], false));
        app.issue(Action::PasteStored);
        app.process_actions();
        assert_eq!(
            app.project()
                .unwrap()
                .items
                .last()
                .unwrap()
                .parent_id
                .as_deref(),
            Some("nested")
        );
        app.settings.update(|p| p.appearance.tree_view = false);
        app.doc_mut().unwrap().folder = Some("folder".into());
        assert_eq!(app.insertion_parent().as_deref(), Some("folder"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn role_bindings_accept_dragged_grid_and_tree_colors_and_undo_together() {
        let (ctx, mut app, root) = app();
        let color = app.project().unwrap().colors().next().unwrap().id.clone();
        let d = app.doc_mut().unwrap();
        d.engine.project.roles.push(Role {
            id: "role".into(),
            name: "text.primary".into(),
            ..Default::default()
        });
        d.engine.project.modes.push(Mode {
            id: "dark".into(),
            name: "Dark".into(),
            overrides: std::collections::BTreeMap::from([("role".into(), String::new())]),
        });
        app.layout = Layout {
            root: Some(Node::Split {
                axis: SplitAxis::Horizontal,
                size: SplitSize::FixedSecond(410.0),
                first: Box::new(Node::Tabs {
                    panels: vec![Panel::Palette],
                    active: 0,
                }),
                second: Box::new(Node::Tabs {
                    panels: vec![Panel::Roles],
                    active: 0,
                }),
            }),
            floating: vec![],
        };
        let values = app.project().unwrap().items.clone();
        let mut tick = 1.0;
        for tree in [false, true] {
            app.settings.update(|s| s.appearance.tree_view = tree);
            for _ in 0..3 {
                frame(&ctx, &mut app, 1440.0, 920.0, tick, vec![]);
                tick += 1.0;
            }
            let from = rect(&ctx, if tree { "test-row" } else { "test-card" }, &color).center();
            let key = if tree { "role:dark" } else { "role:default" };
            let target = ctx
                .data(|d| d.get_temp::<egui::Rect>(Id::new(("test-role-binding", key))))
                .unwrap();
            drag(&ctx, &mut app, from, target.center(), &mut tick, false);
            if tree {
                assert_eq!(app.project().unwrap().modes[0].overrides["role"], color);
            } else {
                assert_eq!(app.project().unwrap().roles[0].color_id, color);
            }
            assert_eq!(app.project().unwrap().items, values);
            app.issue(Action::Undo);
            app.process_actions();
            assert!(app.project().unwrap().roles[0].color_id.is_empty());
            assert!(app.project().unwrap().modes[0].overrides["role"].is_empty());
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn home_editor_panels_and_dialogs_render_at_multiple_sizes_and_themes() {
        let (ctx, mut app, root) = app();
        let id = app.project().unwrap().items[0].id.clone();
        let mut tick = 1.0;
        for dark in [true, false] {
            app.settings
                .update(|p| p.appearance.theme = if dark { "dark" } else { "light" }.into());
            app.apply_theme(&ctx);
            for (w, h) in [(1440.0, 920.0), (1060.0, 720.0)] {
                app.home = true;
                frame(&ctx, &mut app, w, h, tick, vec![]);
                tick += 1.0;
                app.home = false;
                for panel in Panel::ALL {
                    app.issue(Action::ShowPanel(panel));
                    app.process_actions();
                    frame(&ctx, &mut app, w, h, tick, vec![]);
                    tick += 1.0;
                }
                for dialog in [
                    Dialog::Composition(CompositionDraft {
                        originals: vec![id.clone()],
                        ..Default::default()
                    }),
                    Dialog::Conversion {
                        id: id.clone(),
                        target: "lab".into(),
                        assign: false,
                    },
                    Dialog::Documentation {
                        roles: vec![],
                        modes: vec![],
                        relations: vec![],
                    },
                    Dialog::Export {
                        format: Format::Simple,
                        appendix: false,
                        accepted: false,
                    },
                    Dialog::Preferences,
                ] {
                    app.dialog = Some(dialog);
                    frame(&ctx, &mut app, w, h, tick, vec![]);
                    tick += 1.0;
                    assert!(app.error.is_none(), "{:?}", app.error);
                    if matches!(&app.dialog, Some(Dialog::Composition(_))) {
                        frame(&ctx, &mut app, w, h, tick, vec![]);
                        tick += 1.0;
                        let rect = ctx
                            .data(|d| d.get_temp::<egui::Rect>(Id::new("test-composition-modal")))
                            .unwrap();
                        assert!(
                            ctx.content_rect().expand(1.0).contains_rect(rect),
                            "Modal fora da janela: {rect:?}"
                        );
                    }
                    app.dialog = None;
                }
            }
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn document_commands_update_dirty_selection_and_undo() {
        let (_ctx, mut app, root) = app();
        let initial = app.project().unwrap().items.len();
        let item = Item::new("color", None);
        let id = item.id.clone();
        app.issue(Action::Core(Command::Add(item)));
        app.process_actions();
        assert_eq!(app.doc().unwrap().active.as_deref(), Some(id.as_str()));
        assert!(app.doc().unwrap().dirty());
        assert_eq!(app.project().unwrap().items.len(), initial + 1);
        app.issue(Action::Undo);
        app.process_actions();
        assert_eq!(app.project().unwrap().items.len(), initial);
        app.issue(Action::Redo);
        app.process_actions();
        assert_eq!(app.project().unwrap().items.len(), initial + 1);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn automatic_save_updates_project_and_public_snapshot_together() {
        let (_ctx, mut app, root) = app();
        let paths = app.paths.clone().unwrap();
        let path = root.join("autosaved.colorcraft");
        let initial = storage::save(&app.project().unwrap().clone(), &path, None, &paths).unwrap();
        {
            let doc = app.doc_mut().unwrap();
            doc.engine.project = initial.project;
            doc.path = Some(path.clone());
            doc.disk_hash = Some(initial.hash);
            doc.saved_hash = storage::content_hash(&doc.engine.project);
            doc.dirty_cache.replace((u64::MAX, false));
        }
        app.settings.update(|prefs| {
            prefs.general.auto_save_projects = true;
            prefs.general.autosave = false;
        });
        app.issue(Action::Core(Command::Add(Item::new("color", None))));
        app.process_actions();
        assert!(app.doc().unwrap().dirty());

        app.autosave_documents();

        let saved = storage::load(&path).unwrap().project;
        let published = storage::library(&paths).unwrap();
        assert_eq!(saved.count(), 6);
        assert!(!app.doc().unwrap().dirty());
        assert_eq!(published.palettes[0].revision, 2);
        assert_eq!(published.palettes[0].colors, 6);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn clipboard_handles_lists_and_preserves_internal_metadata() {
        let (_ctx, mut app, root) = app();
        let n = app.project().unwrap().count();
        app.paste_text("#123456\nhsl(120 100% 50%)");
        app.process_actions();
        assert_eq!(app.project().unwrap().count(), n + 2);
        let source = app.project().unwrap().clone();
        let id = source.items[0].id.clone();
        let text=serde_json::to_string(&serde_json::json!({"format":"colorcraft-clipboard","project":source,"ids":[id],"cut":false})).unwrap();
        app.paste_text(&text);
        app.process_actions();
        assert_eq!(app.project().unwrap().count(), n + 3);
        assert!(app.error.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn docking_transactions_preserve_document_and_roundtrip_layout() {
        let (ctx, mut app, root) = app();
        let original = app.project().unwrap().clone();
        app.layout
            .apply(docking::Action::Float {
                panel: Panel::Details,
                rect: [20.0, 100.0, 360.0, 500.0],
            })
            .unwrap();
        frame(&ctx, &mut app, 1440.0, 920.0, 1.0, vec![]);
        let encoded = serde_json::to_vec(&app.layout).unwrap();
        let decoded: Layout<Panel> = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(app.layout, decoded);
        assert_eq!(&original, app.project().unwrap());
        for (i, model) in ["HSV", "HSL", "RGB", "OKLCH"].iter().enumerate() {
            app.component_mode = (*model).into();
            app.issue(Action::ShowPanel(Panel::Color));
            app.process_actions();
            frame(&ctx, &mut app, 1440.0, 920.0, i as f64 + 2.0, vec![]);
            assert_eq!(
                &original,
                app.project().unwrap(),
                "Exibir {model} não deve editar valores"
            );
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    fn click(
        ctx: &egui::Context,
        app: &mut ColorCraftApp,
        pos: egui::Pos2,
        modifiers: egui::Modifiers,
        tick: &mut f64,
    ) {
        for pressed in [true, false] {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(1440.0, 920.0),
                    )),
                    time: Some(*tick),
                    focused: true,
                    events: vec![
                        egui::Event::ModifiersChanged(modifiers),
                        egui::Event::PointerMoved(pos),
                        egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers,
                        },
                    ],
                    ..Default::default()
                },
                |ui| app.draw(ui),
            );
            output.textures_delta.clear();
            *tick += 0.15;
        }
    }
    fn rect(ctx: &egui::Context, kind: &str, id: &str) -> egui::Rect {
        ctx.data(|d| d.get_temp(Id::new((kind, id))).unwrap())
    }
    fn panel_rect(ctx: &egui::Context, panel: Panel) -> egui::Rect {
        ctx.data(|d| d.get_temp(Id::new(("test-panel-body", panel))).unwrap())
    }
    fn tab_center(ctx: &egui::Context, panel: Panel) -> egui::Pos2 {
        ctx.read_response(Id::new("colorcraft-workspace").with(("tab", &panel)))
            .unwrap()
            .rect
            .center()
    }
    fn pointer_button(pos: egui::Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        }
    }
    fn drag(
        ctx: &egui::Context,
        app: &mut ColorCraftApp,
        from: egui::Pos2,
        to: egui::Pos2,
        tick: &mut f64,
        escape: bool,
    ) {
        for events in [
            vec![egui::Event::PointerMoved(from), pointer_button(from, true)],
            vec![egui::Event::PointerMoved(from + Vec2::new(9.0, 0.0))],
            vec![egui::Event::PointerMoved(to)],
            if escape {
                vec![egui::Event::Key {
                    key: egui::Key::Escape,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                }]
            } else {
                vec![]
            },
            vec![pointer_button(to, false)],
            vec![],
            vec![],
        ] {
            frame(ctx, app, 1440.0, 920.0, *tick, events);
            *tick += 0.2;
        }
    }
    #[test]
    fn real_tab_drags_float_over_bodies_dock_at_targets_and_cancel_with_escape() {
        let (ctx, mut app, root) = app();
        let original = app.project().unwrap().clone();
        frame(&ctx, &mut app, 1440.0, 920.0, 1.0, vec![]);
        frame(&ctx, &mut app, 1440.0, 920.0, 2.0, vec![]);
        let mut tick = 3.0;
        let source = tab_center(&ctx, Panel::Details);
        let target = panel_rect(&ctx, Panel::Palette).min + Vec2::new(180.0, 180.0);
        drag(&ctx, &mut app, source, target, &mut tick, false);
        assert!(
            app.layout
                .location(&Panel::Details)
                .unwrap()
                .floating
                .is_some(),
            "{:?}",
            app.layout
        );
        // Explicit tab strips still group panels.
        let source = tab_center(&ctx, Panel::Details);
        let target = tab_center(&ctx, Panel::Color);
        drag(&ctx, &mut app, source, target, &mut tick, false);
        assert!(app.layout.floating.is_empty());
        let before = app.layout.clone();
        let source = tab_center(&ctx, Panel::Details);
        let target = panel_rect(&ctx, Panel::Palette).center() + Vec2::new(34.0, 0.0);
        drag(&ctx, &mut app, source, target, &mut tick, true);
        assert_eq!(app.layout, before);
        let source = tab_center(&ctx, Panel::Details);
        drag(&ctx, &mut app, source, target, &mut tick, false);
        assert!(app.layout.floating.is_empty());
        assert_eq!(
            app.layout.location(&Panel::Details).unwrap().anchor,
            Some(Panel::Palette)
        );
        assert_eq!(app.project().unwrap(), &original);
        assert!(app.error.is_none(), "{:?}", app.error);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn headers_have_no_detach_buttons_and_scroll_viewports_fill_the_panel_width() {
        let (ctx, mut app, root) = app();
        for _ in 0..24 {
            app.doc_mut()
                .unwrap()
                .engine
                .project
                .items
                .push(Item::new("color", None));
        }
        for n in 0..20 {
            app.doc_mut().unwrap().engine.project.items[0]
                .custom
                .insert(
                    format!("Um campo personalizado com nome longo {n}"),
                    serde_json::json!("Valor"),
                );
        }
        let original = app.project().unwrap().clone();
        let mut tick = 1.0;
        for (w, h) in [(1440.0, 920.0), (1060.0, 720.0)] {
            for panel in [Panel::Color, Panel::Details, Panel::Links] {
                app.layout
                    .apply(docking::Action::Activate { panel })
                    .unwrap();
                for _ in 0..3 {
                    frame(&ctx, &mut app, w, h, tick, vec![]);
                    tick += 0.2;
                }
                for (panel, id) in [
                    (Panel::Palette, "palette-scroll"),
                    (
                        panel,
                        match panel {
                            Panel::Color => "color-scroll",
                            Panel::Details => "details-scroll",
                            _ => "links-scroll",
                        },
                    ),
                ] {
                    let body = panel_rect(&ctx, panel);
                    let viewport = ctx
                        .data(|d| d.get_temp::<egui::Rect>(Id::new(("test-scroll-viewport", id))))
                        .unwrap();
                    assert!(
                        (viewport.left() - body.left()).abs() < 1.0,
                        "{panel:?}: {body:?} {viewport:?}"
                    );
                    assert!(
                        (body.right() - viewport.right()).abs() <= 14.0,
                        "{panel:?}: scrollbar too far from border: {body:?} {viewport:?}"
                    );
                    if panel == Panel::Details {
                        let size = ctx
                            .data(|d| d.get_temp::<Vec2>(Id::new(("test-scroll-content-size", id))))
                            .unwrap();
                        assert!(size.y > viewport.height());
                        assert!(size.x <= body.width());
                    }
                }
            }
        }
        frame(&ctx, &mut app, 1440.0, 920.0, tick, vec![]);
        assert!(
            ctx.data(|d| d.get_temp::<egui::Rect>(Id::new(("test-detach", Panel::Links))))
                .is_none()
        );
        assert_eq!(app.project().unwrap(), &original);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn technical_badges_and_direct_lock_clicks_preserve_selection_and_colour() {
        let (ctx, mut app, root) = app();
        let ids: Vec<_> = app
            .project()
            .unwrap()
            .colors()
            .take(2)
            .map(|i| i.id.clone())
            .collect();
        app.doc_mut()
            .unwrap()
            .engine
            .execute(Command::LinkColors {
                ids: ids.clone(),
                channels: vec![3],
            })
            .unwrap();
        app.doc_mut().unwrap().selected = vec![ids[1].clone()];
        app.doc_mut().unwrap().active = Some(ids[1].clone());
        app.layout
            .apply(docking::Action::Activate {
                panel: Panel::Links,
            })
            .unwrap();
        let mut tick = 1.0;
        for tree in [false, true] {
            app.settings.update(|p| p.appearance.tree_view = tree);
            frame(&ctx, &mut app, 1440.0, 920.0, tick, vec![]);
            tick += 1.0;
            let badge = rect(&ctx, "test-technical-badge", &ids[0]);
            let item_rect = rect(&ctx, if tree { "test-row" } else { "test-card" }, &ids[0]);
            assert!(item_rect.contains_rect(badge));
            let old = app.project().unwrap().clone();
            let selection = app.doc().unwrap().selected.clone();
            let hand = rect(&ctx, "test-direct-lock", &ids[0]);
            click(&ctx, &mut app, hand.center(), Default::default(), &mut tick);
            assert_eq!(app.doc().unwrap().selected, selection);
            assert_eq!(
                app.project().unwrap().item(&ids[0]).unwrap().rgb,
                old.item(&ids[0]).unwrap().rgb
            );
            assert_eq!(
                app.project().unwrap().item(&ids[0]).unwrap().direct_locked,
                !old.item(&ids[0]).unwrap().direct_locked
            );
            assert!(!app.project().unwrap().item(&ids[0]).unwrap().locked);
            // The badge remains part of the clickable/selectable row or card.
            click(
                &ctx,
                &mut app,
                badge.center(),
                Default::default(),
                &mut tick,
            );
            assert_eq!(app.doc().unwrap().selected, vec![ids[0].clone()]);
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn tree_swatch_and_empty_space_select_and_locks_never_select_or_drag() {
        let (ctx, mut app, root) = app();
        let ids: Vec<_> = app
            .project()
            .unwrap()
            .items
            .iter()
            .map(|i| i.id.clone())
            .collect();
        app.settings.update(|p| p.appearance.tree_view = true);
        frame(&ctx, &mut app, 1440.0, 920.0, 1.0, vec![]);
        frame(&ctx, &mut app, 1440.0, 920.0, 2.0, vec![]);
        let mut tick = 3.0;
        let row = rect(&ctx, "test-row", &ids[1]);
        click(
            &ctx,
            &mut app,
            egui::pos2(row.left() + 42.0, row.center().y),
            Default::default(),
            &mut tick,
        );
        assert_eq!(app.doc().unwrap().selected, vec![ids[1].clone()]);
        let row = rect(&ctx, "test-row", &ids[2]);
        click(
            &ctx,
            &mut app,
            egui::pos2(row.left() + 450.0, row.center().y),
            Default::default(),
            &mut tick,
        );
        assert_eq!(app.doc().unwrap().selected, vec![ids[2].clone()]);
        let row = rect(&ctx, "test-row", &ids[0]);
        click(
            &ctx,
            &mut app,
            row.left_center() + Vec2::new(90.0, 0.0),
            egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            },
            &mut tick,
        );
        assert_eq!(app.doc().unwrap().selected.len(), 2);
        let selection = app.doc().unwrap().selected.clone();
        let old = app.project().unwrap().clone();
        let lock = rect(&ctx, "test-row-lock", &ids[3]);
        click(&ctx, &mut app, lock.center(), Default::default(), &mut tick);
        assert!(app.project().unwrap().item(&ids[3]).unwrap().locked);
        assert_eq!(app.doc().unwrap().selected, selection);
        assert_eq!(
            app.project()
                .unwrap()
                .items
                .iter()
                .map(|i| i.rgb)
                .collect::<Vec<_>>(),
            old.items.iter().map(|i| i.rgb).collect::<Vec<_>>()
        );
        app.settings.update(|p| p.appearance.tree_view = false);
        frame(&ctx, &mut app, 1440.0, 920.0, tick + 1.0, vec![]);
        tick += 2.0;
        let selected = app.doc().unwrap().selected.clone();
        let lock = rect(&ctx, "test-card-lock", &ids[1]);
        click(&ctx, &mut app, lock.center(), Default::default(), &mut tick);
        assert!(app.project().unwrap().item(&ids[1]).unwrap().locked);
        assert_eq!(app.doc().unwrap().selected, selected);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn fixed_levels_hide_in_tree_and_minimum_cards_have_no_inline_controls() {
        let (ctx, mut app, root) = app();
        let id = app.project().unwrap().items[0].id.clone();
        let original = app.project().unwrap().clone();
        app.settings.update(|p| p.appearance.card_zoom = 0.085);
        frame(&ctx, &mut app, 1440.0, 920.0, 1.0, vec![]);
        let small = rect(&ctx, "test-card", &id);
        assert!((small.width() - 96.0 * 5.0 / 25.4).abs() < 0.1);
        assert!((small.height() - small.width()).abs() < 0.1);
        assert!(
            ctx.data(|d| d.get_temp::<egui::Rect>(Id::new(("test-card-lock", &id))))
                .is_none()
        );
        let rendered = ctx
            .data(|d| d.get_temp::<Vec<String>>(Id::new("test-rendered-text")))
            .unwrap();
        assert!(rendered.iter().any(|s| s.contains("Mínimo")));
        app.settings.update(|p| p.appearance.tree_view = true);
        frame(&ctx, &mut app, 1440.0, 920.0, 2.0, vec![]);
        let rendered = ctx
            .data(|d| d.get_temp::<Vec<String>>(Id::new("test-rendered-text")))
            .unwrap();
        assert!(!rendered.iter().any(|s| s.contains("Mínimo")));
        app.settings.update(|p| p.appearance.tree_view = false);
        frame(&ctx, &mut app, 1440.0, 920.0, 3.0, vec![]);
        assert_eq!(app.settings.get().appearance.card_zoom, 0.085);
        assert_eq!(app.project().unwrap(), &original);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn picking_preserves_draft_and_blocks_mutations_then_escape_restores_modal() {
        let (ctx, mut app, root) = app();
        let p = app.project().unwrap().clone();
        let a = p.items[0].id.clone();
        let b = p.items[1].id.clone();
        app.dialog = Some(Dialog::Composition(CompositionDraft {
            originals: vec![a],
            spread: 45.0,
            ..Default::default()
        }));
        app.issue(Action::BeginPick(0));
        app.process_actions();
        assert!(app.picking.is_some());
        app.issue(Action::New);
        app.issue(Action::Core(Command::Delete(vec![b.clone()])));
        app.process_actions();
        assert_eq!(app.docs.len(), 1);
        assert_eq!(app.project().unwrap(), &p);
        app.issue(Action::ChooseReplacement(b.clone()));
        app.process_actions();
        let Some(Dialog::Composition(d)) = &app.dialog else {
            panic!("Modal não restaurado")
        };
        assert_eq!(d.spread, 45.0);
        assert_eq!(d.replacements[&0].id, b);
        app.issue(Action::BeginPick(1));
        app.process_actions();
        frame(
            &ctx,
            &mut app,
            1440.0,
            920.0,
            3.0,
            vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
        );
        assert!(app.picking.is_none());
        assert!(matches!(app.dialog, Some(Dialog::Composition(_))));
        assert_eq!(app.project().unwrap(), &p);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn inline_rename_confirms_cancels_and_preserves_color_data() {
        let (ctx, mut app, root) = app();
        let old = app.project().unwrap().clone();
        app.title_edit = Some(crate::widgets::TitleEdit {
            project: old.id.clone(),
            item: None,
            value: "Paleta renomeada".into(),
            focus: true,
        });
        frame(&ctx, &mut app, 1440.0, 920.0, 1.0, vec![]);
        frame(
            &ctx,
            &mut app,
            1440.0,
            920.0,
            2.0,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
        );
        assert_eq!(app.project().unwrap().name, "Paleta renomeada");
        assert_eq!(app.project().unwrap().items, old.items);
        app.issue(Action::Undo);
        app.process_actions();
        assert_eq!(app.project().unwrap(), &old);
        let item = old.items[0].id.clone();
        app.title_edit = Some(crate::widgets::TitleEdit {
            project: old.id.clone(),
            item: Some(item),
            value: "Cancelar nome".into(),
            focus: true,
        });
        frame(&ctx, &mut app, 1440.0, 920.0, 3.0, vec![]);
        frame(
            &ctx,
            &mut app,
            1440.0,
            920.0,
            4.0,
            vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
        );
        assert!(app.title_edit.is_none());
        assert_eq!(app.project().unwrap(), &old);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn value_bar_changes_only_hsv_value_and_respects_lock() {
        let (ctx, mut app, root) = app();
        let id = app.project().unwrap().items[0].id.clone();
        let original = app.project().unwrap().clone();
        frame(&ctx, &mut app, 1440.0, 920.0, 1.0, vec![]);
        frame(&ctx, &mut app, 1440.0, 920.0, 2.0, vec![]);
        let bar = ctx
            .data(|d| d.get_temp::<egui::Rect>(Id::new("test-value-bar")))
            .unwrap();
        let mut tick = 3.0;
        click(
            &ctx,
            &mut app,
            egui::pos2(
                bar.left() + 9.0 + (bar.width() - 18.0) * 0.5,
                bar.center().y,
            ),
            Default::default(),
            &mut tick,
        );
        let actual = colorcraft_core::color::hsv(app.project().unwrap().item(&id).unwrap().rgb);
        let before = colorcraft_core::color::hsv(original.item(&id).unwrap().rgb);
        assert!((actual[0] - before[0]).abs() < 1e-5);
        assert!((actual[1] - before[1]).abs() < 1e-5);
        assert!((actual[2] - 0.5).abs() < 0.01);
        app.issue(Action::Undo);
        app.process_actions();
        assert_eq!(app.project().unwrap(), &original);
        app.issue(Action::Core(Command::Lock { id, locked: true }));
        app.process_actions();
        let locked = app.project().unwrap().clone();
        frame(&ctx, &mut app, 1440.0, 920.0, tick + 1.0, vec![]);
        tick += 2.0;
        click(&ctx, &mut app, bar.center(), Default::default(), &mut tick);
        assert_eq!(app.project().unwrap(), &locked);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn multi_link_dialog_keeps_every_selected_colour_and_expanded_tree_ranges() {
        let (_ctx, mut app, root) = app();
        let ids: Vec<_> = app
            .project()
            .unwrap()
            .colors()
            .take(3)
            .map(|i| i.id.clone())
            .collect();
        app.doc_mut().unwrap().selected = ids.clone();
        app.open_link();
        app.process_actions();
        assert!(matches!(&app.dialog,Some(Dialog::Link{participants,..}) if participants==&ids));
        let folder = Item::new("folder", None);
        let folder_id = folder.id.clone();
        app.doc_mut()
            .unwrap()
            .engine
            .execute(Command::Add(folder))
            .unwrap();
        app.doc_mut()
            .unwrap()
            .engine
            .execute(Command::Move {
                ids: ids[0..2].to_vec(),
                parent: Some(folder_id.clone()),
            })
            .unwrap();
        app.doc_mut().unwrap().expanded.insert(folder_id.clone());
        app.settings.update(|p| p.appearance.tree_view = true);
        let visible = app.doc().unwrap().visible_item_ids(true);
        let start = visible.iter().position(|i| i == &folder_id).unwrap();
        assert_eq!(visible[start + 1..start + 3], ids[0..2]);
        app.select(&folder_id, false, false);
        app.select(&ids[1], false, true);
        assert_eq!(
            app.doc().unwrap().selected,
            vec![folder_id, ids[0].clone(), ids[1].clone()]
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
