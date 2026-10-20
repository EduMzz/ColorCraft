//! Native storage and publication. No network I/O and no UI dependencies.
use colorcraft_core::{
    export::{self, Format},
    *,
};
use craft_settings::{Location, SettingsStore};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
    time::SystemTime,
};

pub fn now() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into())
}
pub fn hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}
pub fn content_hash(p: &Project) -> String {
    let mut p = p.clone();
    p.revision = 0;
    p.updated_at.clear();
    p.created_at.clear();
    hash(&serde_json::to_vec(&p).unwrap_or_default())
}
pub fn bounded_read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let file =
        File::open(path).map_err(|e| format!("Não foi possível abrir {}: {e}", path.display()))?;
    if file.metadata().map_err(|e| e.to_string())?.len() > limit {
        return Err(format!(
            "Arquivo excede o limite de {} MiB.",
            limit / 1024 / 1024
        ));
    }
    let mut bytes = vec![];
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("Arquivo excede o limite permitido.".into());
    }
    Ok(bytes)
}
#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Appearance {
    pub theme: String,
    pub language: String,
    pub scale: f32,
    pub card_zoom: f32,
    pub tree_view: bool,
}
impl Default for Appearance {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            language: "pt-BR".into(),
            scale: 1.0,
            card_zoom: 1.0,
            tree_view: false,
        }
    }
}
#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct General {
    pub autosave: bool,
    pub auto_save_projects: bool,
    pub library_dir: String,
}
impl Default for General {
    fn default() -> Self {
        Self {
            autosave: true,
            auto_save_projects: true,
            library_dir: String::new(),
        }
    }
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Recent {
    pub path: String,
    pub name: String,
    pub colors: usize,
    pub preview: Vec<[f64; 3]>,
    pub updated_at: String,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Files {
    pub recent: Vec<Recent>,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Prefs {
    pub appearance: Appearance,
    pub general: General,
    pub files: Files,
}
impl craft_settings::Settings for Prefs {
    const APP_ID: &'static str = "colorcraft";
    const SCHEMA_VERSION: u32 = 1;
    const PORTABLE_MARKERS: &'static [&'static str] = &["portable", "portable.txt"];
    const PORTABLE_DIR: &'static str = "ColorCraftData/config";
}
pub fn open_settings(dir: Option<PathBuf>, isolated: bool) -> (SettingsStore<Prefs>, String) {
    let location = if isolated {
        Location::Memory
    } else if let Some(path) = dir {
        Location::Dir(path)
    } else {
        Location::Auto
    };
    let (store, report) = SettingsStore::open(location);
    let message = if report.has_problems() {
        format!("Preferências: {report:?}")
    } else {
        String::new()
    };
    (store, message)
}
#[derive(Clone, Debug)]
pub struct Paths {
    pub config: Option<PathBuf>,
    pub data: PathBuf,
    pub library: PathBuf,
    pub recovery: PathBuf,
    legacy_library: Option<PathBuf>,
    migrate_legacy_libraries: bool,
}
impl Paths {
    pub fn new(store: &SettingsStore<Prefs>, override_data: Option<PathBuf>) -> Result<Self> {
        let paths = Self::resolve(store, override_data)?;
        paths.prepare()?;
        Ok(paths)
    }
    pub fn resolve(store: &SettingsStore<Prefs>, override_data: Option<PathBuf>) -> Result<Self> {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let portable = exe
            .parent()
            .filter(|dir| dir.join("portable").exists() || dir.join("portable.txt").exists());
        let isolated = override_data.is_some();
        let data = override_data.unwrap_or_else(|| {
            if let Some(dir) = portable {
                dir.join("ColorCraftData")
            } else {
                std::env::var_os("LOCALAPPDATA")
                    .map(PathBuf::from)
                    .or_else(|| std::env::var_os("XDG_DATA_HOME").map(PathBuf::from))
                    .unwrap_or_else(std::env::temp_dir)
                    .join("colorcraft")
            }
        });
        // QA/isolated runs keep their library beside their temporary data. Every
        // real installation publishes to the same per-user ArtCraft location,
        // regardless of whether the executable itself is portable.
        let library = if isolated {
            if store.get().general.library_dir.is_empty() {
                data.join("library")
            } else {
                PathBuf::from(&store.get().general.library_dir)
            }
        } else {
            std::env::var_os("APPDATA")
                .map(PathBuf::from)
                .or_else(|| std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from))
                .unwrap_or_else(std::env::temp_dir)
                .join("ArtCraft")
                .join("Palettes")
        };
        let paths = Self {
            config: store.dir().map(Path::to_path_buf),
            recovery: data.join("recovery"),
            legacy_library: (!store.get().general.library_dir.is_empty())
                .then(|| PathBuf::from(&store.get().general.library_dir)),
            migrate_legacy_libraries: !isolated,
            data,
            library,
        };
        Ok(paths)
    }
    pub fn prepare(&self) -> Result<()> {
        fs::create_dir_all(&self.recovery).map_err(|e| e.to_string())?;
        fs::create_dir_all(&self.library).map_err(|e| e.to_string())?;
        if self.migrate_legacy_libraries {
            merge_legacy_libraries(self)?;
        }
        ensure_library_index(&self.library)?;
        let descriptor = serde_json::json!({"schemaVersion":1,"format":"colorcraft-library-location","libraryRoot":self.library,"index":"library.json"});
        atomic(
            &self.data.join("library-location.json"),
            &serde_json::to_vec_pretty(&descriptor).map_err(|e| e.to_string())?,
        )?;
        Ok(())
    }
}

fn ensure_library_index(root: &Path) -> Result<()> {
    let _guard = lock(&root.join("library.lock"))?;
    let index = root.join("library.json");
    if !index.exists() {
        atomic(
            &index,
            &serde_json::to_vec_pretty(&Library {
                schema_version: 1,
                palettes: vec![],
            })
            .map_err(|e| e.to_string())?,
        )?;
    }
    Ok(())
}

fn merge_legacy_libraries(paths: &Paths) -> Result<()> {
    let mut candidates = vec![];
    if let Some(legacy) = &paths.legacy_library {
        candidates.push(legacy.clone());
    }
    if paths.library != paths.data.join("library") {
        candidates.push(paths.data.join("library"));
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let legacy_data = PathBuf::from(local).join("colorcraft");
        candidates.push(legacy_data.join("library"));
        candidates.extend(descriptor_library_roots(
            &legacy_data.join("library-location.json"),
        ));
    }
    if let Some(roaming) = std::env::var_os("APPDATA") {
        let old_roaming = PathBuf::from(roaming).join("colorcraft");
        candidates.push(old_roaming.join("library"));
        candidates.extend(descriptor_library_roots(
            &old_roaming.join("library-location.json"),
        ));
    }
    if let Some(custom) = std::env::var_os("LOCALAPPDATA") {
        // Releases before the canonical location used per-install descriptors;
        // resolve those explicitly without walking arbitrary folders.
        let legacy_data = PathBuf::from(custom).join("colorcraft");
        candidates.extend(descriptor_library_roots(
            &legacy_data.join("library-location.json"),
        ));
    }
    candidates.sort();
    candidates.dedup();
    for source in candidates {
        if same_directory(&source, &paths.library) || !source.join("library.json").is_file() {
            continue;
        }
        merge_library(&source, &paths.library)?;
    }
    Ok(())
}

fn descriptor_library_roots(descriptor: &Path) -> Vec<PathBuf> {
    let Ok(bytes) = bounded_read(descriptor, 64 * 1024) else {
        return vec![];
    };
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return vec![];
    };
    value
        .get("libraryRoot")
        .and_then(serde_json::Value::as_str)
        .map(PathBuf::from)
        .into_iter()
        .collect()
}

fn same_directory(left: &Path, right: &Path) -> bool {
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

fn merge_library(source: &Path, destination: &Path) -> Result<()> {
    let source_bytes = bounded_read(&source.join("library.json"), 8 * 1024 * 1024)?;
    let source_index: Library = serde_json::from_slice(&source_bytes)
        .map_err(|e| format!("Índice antigo de biblioteca inválido: {e}"))?;
    if source_index.schema_version != 1 {
        return Err("Biblioteca antiga usa uma versão não suportada; os arquivos originais foram preservados.".into());
    }

    let _guard = lock(&destination.join("library.lock"))?;
    let mut target = if destination.join("library.json").exists() {
        let bytes = bounded_read(&destination.join("library.json"), 8 * 1024 * 1024)?;
        let index: Library = serde_json::from_slice(&bytes)
            .map_err(|e| format!("Índice canônico de biblioteca inválido: {e}"))?;
        if index.schema_version != 1 {
            return Err(
                "Biblioteca canônica usa uma versão não suportada; nenhum arquivo foi sobrescrito."
                    .into(),
            );
        }
        index
    } else {
        Library {
            schema_version: 1,
            palettes: vec![],
        }
    };
    let mut changed = false;
    for entry in source_index.palettes {
        let relative = Path::new(&entry.snapshot);
        if relative.as_os_str().is_empty()
            || relative.is_absolute()
            || relative
                .components()
                .any(|c| !matches!(c, std::path::Component::Normal(_)))
        {
            return Err("Biblioteca antiga contém um caminho de captura inválido; os originais foram preservados.".into());
        }
        let source_snapshot = source.join(relative);
        let source_bytes = bounded_read(&source_snapshot, 256 * 1024 * 1024)?;
        let snapshot_value: serde_json::Value =
            serde_json::from_slice(&source_bytes).map_err(|e| {
                format!(
                    "Captura antiga inválida em {}: {e}",
                    source_snapshot.display()
                )
            })?;
        export::from_palette(&source_bytes).map_err(|e| {
            format!(
                "Captura antiga inválida em {}: {e}",
                source_snapshot.display()
            )
        })?;
        if snapshot_value
            .get("paletteId")
            .and_then(serde_json::Value::as_str)
            != Some(entry.palette_id.as_str())
            || snapshot_value
                .get("revision")
                .and_then(serde_json::Value::as_u64)
                != Some(entry.revision)
        {
            return Err(
                "Captura antiga não corresponde ao índice; os originais foram preservados.".into(),
            );
        }
        let target_snapshot = destination.join(relative);
        if target_snapshot.exists() {
            let existing = bounded_read(&target_snapshot, 256 * 1024 * 1024)?;
            if existing != source_bytes {
                return Err(format!(
                    "Conflito ao migrar a paleta ‘{}’: já existe uma captura diferente no destino. Os originais foram preservados.",
                    entry.name
                ));
            }
        } else {
            if let Some(parent) = target_snapshot.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            craft_atomic::write_new(&target_snapshot, &source_bytes).map_err(|e| e.to_string())?;
        }
        match target
            .palettes
            .iter_mut()
            .find(|current| current.palette_id == entry.palette_id)
        {
            Some(current) if current.revision < entry.revision => {
                *current = entry;
                changed = true;
            }
            Some(current)
                if current.revision == entry.revision && current.snapshot != entry.snapshot =>
            {
                return Err(format!(
                    "Conflito de revisão ao migrar a paleta ‘{}’. Os originais foram preservados.",
                    entry.name
                ));
            }
            Some(_) => {}
            None => {
                target.palettes.push(entry);
                changed = true;
            }
        }
    }
    if changed || !destination.join("library.json").exists() {
        target.schema_version = 1;
        atomic(
            &destination.join("library.json"),
            &serde_json::to_vec_pretty(&target).map_err(|e| e.to_string())?,
        )?;
    }
    Ok(())
}
pub fn atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    craft_atomic::atomic_write(path, bytes)
        .map_err(|e| format!("Falha ao gravar {}: {e}", path.display()))
}
fn private_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    craft_atomic::WriteOptions::default()
        .with_new_file_permissions(craft_atomic::NewFilePermissions::Private)
        .atomic_write(path, bytes)
        .map_err(|e| e.to_string())
}
pub struct Loaded {
    pub project: Project,
    pub hash: String,
    pub imported: bool,
}
pub fn load(path: &Path) -> Result<Loaded> {
    let bytes = bounded_read(path, 256 * 1024 * 1024)?;
    let digest = hash(&bytes);
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    let (p, imported) = if ext == "gpl" {
        (import_gpl(&bytes)?, true)
    } else if ext == "color-palette" {
        (import_simple(&bytes)?, true)
    } else {
        let v: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|e| format!("Arquivo JSON inválido: {e}"))?;
        if v.get("format").and_then(|x| x.as_str()) == Some("colorcraft-palette") {
            (export::from_palette(&bytes)?, true)
        } else {
            if !v.get("items").is_some_and(|x| x.is_array()) {
                return Err(
                    "O arquivo não contém um projeto ColorCraft/Colorator reconhecível.".into(),
                );
            }
            (Project::from_json(&bytes)?, ext != "colorcraft")
        }
    };
    let bad = colorcraft_core::solver::problems(&p, &colorcraft_core::solver::state(&p), None);
    if !bad.is_empty() {
        return Err(format!(
            "O arquivo contém relações incompatíveis ({}). O original foi preservado.",
            bad.join(", ")
        ));
    }
    Ok(Loaded {
        project: p,
        hash: digest,
        imported,
    })
}
fn lock(path: &Path) -> Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.try_lock_exclusive()
        .map_err(|_| "Há outra gravação em andamento. Tente novamente em instantes.".to_string())?;
    Ok(file)
}
pub struct Saved {
    pub project: Project,
    pub hash: String,
    pub publication_error: Option<String>,
}
pub fn save(
    project: &Project,
    path: &Path,
    expected: Option<&str>,
    paths: &Paths,
) -> Result<Saved> {
    project.validate()?;
    let parent = path.parent().ok_or("Destino inválido")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let _guard = lock(&path.with_extension("colorcraft.lock"))?;
    let mut old_revision = 0;
    if path.exists() {
        let old = bounded_read(path, 256 * 1024 * 1024)?;
        if expected != Some(hash(&old).as_str()) {
            return Err("Este arquivo mudou em outro processo. Salve uma cópia ou reabra a versão atual; nenhuma alteração foi sobrescrita.".into());
        }
        if let Ok(oldp) = Project::from_json(&old) {
            old_revision = oldp.revision;
        }
        atomic(&path.with_extension("colorcraft.previous"), &old)?;
    } else if expected.is_some() {
        return Err("O arquivo original foi removido ou movido. Use Salvar como para escolher um novo destino.".into());
    }
    let mut p = project.clone();
    p.schema_version = 4;
    p.revision = old_revision
        .max(project.revision)
        .checked_add(1)
        .ok_or("Revisão excedida")?;
    p.updated_at = now();
    if p.created_at.is_empty() {
        p.created_at = p.updated_at.clone()
    }
    let bytes = serde_json::to_vec_pretty(&p).map_err(|e| e.to_string())?;
    if bytes.len() > 256 * 1024 * 1024 {
        return Err("O projeto salvo excede 256 MiB. Incorpore menos fontes ou mantenha arquivos grandes vinculados.".into());
    }
    atomic(path, &bytes)?;
    let digest = hash(&bytes);
    let publication_error = publish(&p, path, paths).err();
    let _ = fs::remove_file(paths.recovery.join(format!("{}.json", safe_id(&p.id))));
    Ok(Saved {
        project: p,
        hash: digest,
        publication_error,
    })
}
fn safe_id(id: &str) -> String {
    hash(id.as_bytes()).chars().take(32).collect()
}
#[derive(Clone, Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct LibraryEntry {
    pub palette_id: String,
    pub revision: u64,
    pub name: String,
    pub colors: usize,
    pub updated_at: String,
    pub snapshot: String,
    pub source_path: String,
    pub preview: Vec<[f64; 3]>,
}
#[derive(Clone, Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Library {
    pub schema_version: u32,
    pub palettes: Vec<LibraryEntry>,
}
pub fn library(paths: &Paths) -> Result<Library> {
    let path = paths.library.join("library.json");
    if !path.exists() {
        return Ok(Library {
            schema_version: 1,
            palettes: vec![],
        });
    }
    let bytes = bounded_read(&path, 8 * 1024 * 1024)?;
    let lib: Library = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Índice de biblioteca inválido: {e}"))?;
    if lib.schema_version != 1 {
        return Err(
            "Biblioteca usa uma versão não suportada. Nenhum índice foi sobrescrito.".into(),
        );
    }
    Ok(lib)
}
pub fn publish(p: &Project, source: &Path, paths: &Paths) -> Result<()> {
    let _guard = lock(&paths.library.join("library.lock"))?;
    let mut lib = library(paths)?;
    let bytes = export::encode(p, Format::Palette, false)?;
    let digest = hash(&bytes);
    let folder = paths.library.join("snapshots").join(safe_id(&p.id));
    fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let file = format!("{}-{}.colorcraft-palette", p.revision, &digest[..16]);
    let snapshot = folder.join(&file);
    if !snapshot.exists() {
        craft_atomic::write_new(&snapshot, &bytes).map_err(|e| e.to_string())?;
    }
    let relative = format!("snapshots/{}/{}", safe_id(&p.id), file);
    let entry = LibraryEntry {
        palette_id: p.id.clone(),
        revision: p.revision,
        name: p.name.clone(),
        colors: p.count(),
        updated_at: p.updated_at.clone(),
        snapshot: relative,
        source_path: source.to_string_lossy().into_owned(),
        preview: p.colors().take(8).map(|i| i.rgb).collect(),
    };
    if let Some(current) = lib.palettes.iter_mut().find(|e| e.palette_id == p.id) {
        if current.revision > p.revision {
            return Err("A biblioteca contém revisão mais nova desta paleta; o projeto foi salvo e a publicação ficou pendente.".into());
        }
        *current = entry
    } else {
        lib.palettes.push(entry)
    }
    lib.schema_version = 1;
    atomic(
        &paths.library.join("library.json"),
        &serde_json::to_vec_pretty(&lib).map_err(|e| e.to_string())?,
    )
}
pub fn recover(project: &Project, paths: &Paths) -> Result<()> {
    let bytes = serde_json::to_vec(project).map_err(|e| e.to_string())?;
    if bytes.len() > 256 * 1024 * 1024 {
        return Err(
            "A recuperação automática excede 256 MiB. Mantenha fontes grandes vinculadas.".into(),
        );
    }
    private_atomic(
        &paths
            .recovery
            .join(format!("{}.json", safe_id(&project.id))),
        &bytes,
    )
}
pub fn recoveries(paths: &Paths) -> Vec<PathBuf> {
    fs::read_dir(&paths.recovery)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == "json"))
                .collect()
        })
        .unwrap_or_default()
}
pub fn discard_recovery(id: &str, paths: &Paths) -> Result<()> {
    let path = paths.recovery.join(format!("{}.json", safe_id(id)));
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
pub fn export_file(p: &Project, path: &Path, f: Format, appendix: bool) -> Result<()> {
    let bytes = export::encode(p, f, appendix)?;
    atomic(path, &bytes)
}
pub fn recent(p: &Project, path: &Path) -> Recent {
    Recent {
        path: path.to_string_lossy().into_owned(),
        name: p.name.clone(),
        colors: p.count(),
        preview: p.colors().take(8).map(|i| i.rgb).collect(),
        updated_at: now(),
    }
}
pub fn file_hash(path: &Path) -> Result<String> {
    bounded_read(path, 256 * 1024 * 1024).map(|bytes| hash(&bytes))
}
pub fn import_profile(path: &Path) -> Result<IccProfile> {
    let bytes = bounded_read(path, 4 * 1024 * 1024)?;
    let profile = craft_profile(&bytes)?;
    Ok(IccProfile {
        id: hash(&bytes),
        name: profile,
        bytes,
    })
}

fn craft_profile(bytes: &[u8]) -> Result<String> {
    // Parsing stays in the pure colour core.
    colorcraft_core::profile_name(bytes)
}
fn import_gpl(bytes: &[u8]) -> Result<Project> {
    let text = std::str::from_utf8(bytes).map_err(|_| "GPL precisa ser UTF-8")?;
    if !text.starts_with("GIMP Palette") {
        return Err("Cabeçalho GPL inválido.".into());
    }
    let mut p = Project::default();
    for line in text.lines().skip(1) {
        let line = line.trim();
        if let Some(name) = line.strip_prefix("Name:") {
            p.name = name.trim().into();
            continue;
        }
        if line.is_empty() || line.starts_with('#') || line.starts_with("Columns:") {
            continue;
        }
        let mut words = line.split_whitespace();
        let mut rgb = [0.0; 3];
        for c in &mut rgb {
            *c = words
                .next()
                .ok_or("Linha GPL incompleta")?
                .parse::<u8>()
                .map_err(|_| "Canal GPL inválido")? as f64;
        }
        let name = words.collect::<Vec<_>>().join(" ");
        p.items.push(Item {
            rgb,
            name,
            ..Item::default()
        });
        if p.items.len() > 10000 {
            return Err("Paleta excede 10.000 cores.".into());
        }
    }
    p.validate()?;
    Ok(p)
}
fn import_simple(bytes: &[u8]) -> Result<Project> {
    let v: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    let colors = v
        .get("colors")
        .and_then(|x| x.as_array())
        .ok_or("Paleta sem cores")?;
    let mut p = Project {
        name: v
            .get("name")
            .and_then(|x| x.as_str())
            .unwrap_or("Paleta importada")
            .into(),
        ..Project::default()
    };
    for col in colors {
        let components = col
            .get("components")
            .and_then(|x| x.as_array())
            .ok_or("Componentes ausentes")?;
        if components.len() != 3 {
            return Err("Esta versão importa apenas cores opacas. Transparência foi preservada no arquivo original; importação cancelada.".into());
        }
        let mut rgb = [0.0; 3];
        for (i, c) in components.iter().enumerate() {
            rgb[i] =
                colorcraft_core::color::gamma(c.as_f64().ok_or("Componente inválido")?) * 255.0;
        }
        if !colorcraft_core::color::in_gamut(rgb) {
            return Err("A paleta usa sRGB estendido fora do gamut. A importação foi cancelada para evitar uma conversão silenciosa.".into());
        }
        p.items.push(Item {
            rgb,
            name: col
                .get("name")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .into(),
            ..Item::default()
        });
    }
    p.validate()?;
    Ok(p)
}
pub fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod library_migration_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn legacy_library_merge_is_validated_and_idempotent() {
        let root = std::env::temp_dir().join(format!("colorcraft-library-merge-{}", uid()));
        let source_root = root.join("legacy");
        let target_root = root.join("canonical");
        fs::create_dir_all(&source_root).unwrap();
        fs::create_dir_all(&target_root).unwrap();
        let (settings, _) = open_settings(None, true);
        let source_paths = Paths::resolve(&settings, Some(source_root.clone())).unwrap();
        source_paths.prepare().unwrap();
        let saved = save(
            &Project::demo(),
            &root.join("palette.colorcraft"),
            None,
            &source_paths,
        )
        .unwrap();
        let before = library(&source_paths).unwrap();

        merge_library(&source_paths.library, &target_root).unwrap();
        merge_library(&source_paths.library, &target_root).unwrap();

        let migrated = library(&Paths {
            config: None,
            data: root.join("target-data"),
            library: target_root.clone(),
            recovery: root.join("target-recovery"),
            legacy_library: None,
            migrate_legacy_libraries: false,
        })
        .unwrap();
        assert_eq!(migrated.palettes.len(), 1);
        assert_eq!(migrated.palettes[0].palette_id, saved.project.id);
        assert_eq!(migrated.palettes[0].revision, before.palettes[0].revision);
        assert!(target_root.join(&migrated.palettes[0].snapshot).is_file());
        fs::remove_dir_all(root).unwrap();
    }
}
