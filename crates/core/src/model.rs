use crate::{
    Result,
    color::{self, Rgb},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
pub fn uid() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn schema() -> u32 {
    4
}
fn default_rgb() -> Rgb {
    [126.0, 111.0, 239.0]
}
fn color_kind() -> String {
    "color".into()
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct NativeColor {
    pub space: String,
    pub components: Vec<f64>,
    pub profile_id: Option<String>,
}
impl Default for NativeColor {
    fn default() -> Self {
        Self::srgb(default_rgb())
    }
}
impl NativeColor {
    pub fn srgb(rgb: Rgb) -> Self {
        Self {
            space: "srgb".into(),
            components: rgb.map(|x| x / 255.0).to_vec(),
            profile_id: None,
        }
    }
    pub fn is_srgb(&self) -> bool {
        self.space == "srgb" && self.profile_id.is_none()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct IccProfile {
    pub id: String,
    pub name: String,
    pub bytes: Vec<u8>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Item {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub css_name: String,
    pub description: String,
    pub usage: String,
    pub tags: Vec<String>,
    pub custom: BTreeMap<String, Value>,
    pub rgb: Rgb,
    pub locked: bool,
    /// Locked because a source capture owns the sampled value.
    #[serde(skip_serializing_if = "is_false")]
    pub source_locked: bool,
    /// Prevent direct editing without pinning this colour in the relationship solver.
    #[serde(skip_serializing_if = "is_false")]
    pub direct_locked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<NativeColor>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}
impl Default for Item {
    fn default() -> Self {
        Self {
            id: uid(),
            kind: color_kind(),
            parent_id: None,
            name: String::new(),
            css_name: String::new(),
            description: String::new(),
            usage: String::new(),
            tags: vec![],
            custom: BTreeMap::new(),
            rgb: default_rgb(),
            locked: false,
            source_locked: false,
            direct_locked: false,
            color: None,
            extra: BTreeMap::new(),
        }
    }
}
fn is_false(value: &bool) -> bool {
    !*value
}
impl Item {
    pub fn new(kind: &str, parent: Option<String>) -> Self {
        Self {
            kind: kind.into(),
            parent_id: parent,
            name: if kind == "folder" {
                "Nova pasta".into()
            } else {
                "Nova cor".into()
            },
            ..Self::default()
        }
    }
    pub fn is_color(&self) -> bool {
        self.kind == "color"
    }
    pub fn native(&self) -> NativeColor {
        self.color
            .clone()
            .unwrap_or_else(|| NativeColor::srgb(self.rgb))
    }
    pub fn is_srgb(&self) -> bool {
        self.color.as_ref().is_none_or(NativeColor::is_srgb)
    }
    pub fn set_rgb(&mut self, rgb: Rgb) {
        self.rgb = rgb;
        if self.color.is_some() {
            self.color = Some(NativeColor::srgb(rgb))
        }
    }
    pub fn label(&self) -> String {
        if self.name.trim().is_empty() {
            if self.is_color() {
                color::hex(self.rgb)
            } else {
                "Pasta".into()
            }
        } else {
            self.name.clone()
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Relation {
    pub mode: String,
    pub value: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Link {
    pub id: String,
    pub a: String,
    pub b: String,
    pub channels: Vec<usize>,
    pub relations: BTreeMap<String, Relation>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Constraint {
    pub a: String,
    pub b: String,
    pub offset: [f64; 3],
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Harmony {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub ids: Vec<String>,
    pub anchor_id: Option<String>,
    pub spread: Option<f64>,
    pub constraints: Vec<Constraint>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Gradient {
    pub id: String,
    pub a: String,
    pub b: String,
    pub ids: Vec<String>,
    pub count: usize,
    pub space: String,
    pub detached: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Limit {
    pub id: String,
    pub a: String,
    pub b: String,
    pub minimum: f64,
    pub metric: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Documentary {
    pub id: String,
    pub from: String,
    pub to: String,
    pub kind: String,
    pub note: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub color_id: String,
    pub description: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Mode {
    pub id: String,
    pub name: String,
    pub overrides: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct CaptureStroke {
    /// Points are normalized to the rendered page/image dimensions.
    pub points: Vec<[f32; 2]>,
    /// Brush radius in source pixels.
    pub radius: f32,
}
impl Default for CaptureStroke {
    fn default() -> Self {
        Self {
            points: vec![],
            radius: 8.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum CaptureMethod {
    #[default]
    Pixel,
    Area {
        shape: String,
        statistic: String,
        radius: u16,
    },
    Brush {
        statistic: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ColorCapture {
    pub id: String,
    pub name: String,
    /// A linked palette colour. None means this capture has been detached.
    pub color_id: Option<String>,
    pub page: u32,
    pub method: CaptureMethod,
    /// Normalized source position for pixel and neighbourhood capture.
    pub position: [f32; 2],
    pub strokes: Vec<CaptureStroke>,
    pub value: Rgb,
}
impl Default for ColorCapture {
    fn default() -> Self {
        Self {
            id: uid(),
            name: "Captura".into(),
            color_id: None,
            page: 0,
            method: CaptureMethod::default(),
            position: [0.5, 0.5],
            strokes: vec![],
            value: default_rgb(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct SourceAsset {
    pub id: String,
    pub name: String,
    pub path: String,
    /// `linked` or `embedded`; linked files remain at their original path.
    pub storage: String,
    pub fingerprint: String,
    pub embedded_data: Option<std::sync::Arc<String>>,
    /// Small last-known preview, retained so a missing link still has a thumbnail.
    pub preview_png: Vec<u8>,
    pub page_count: u32,
    pub captures: Vec<ColorCapture>,
}
impl Default for SourceAsset {
    fn default() -> Self {
        Self {
            id: uid(),
            name: String::new(),
            path: String::new(),
            storage: "linked".into(),
            fingerprint: String::new(),
            embedded_data: None,
            preview_png: vec![],
            page_count: 1,
            captures: vec![],
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Project {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub description: String,
    pub revision: u64,
    pub items: Vec<Item>,
    pub links: Vec<Link>,
    pub harmonies: Vec<Harmony>,
    pub gradients: Vec<Gradient>,
    pub limits: Vec<Limit>,
    pub profiles: Vec<IccProfile>,
    pub documentary_relations: Vec<Documentary>,
    pub roles: Vec<Role>,
    pub modes: Vec<Mode>,
    pub sources: Vec<SourceAsset>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}
impl Default for Project {
    fn default() -> Self {
        Self {
            schema_version: schema(),
            id: uid(),
            name: "Nova paleta".into(),
            description: String::new(),
            revision: 0,
            items: vec![],
            links: vec![],
            harmonies: vec![],
            gradients: vec![],
            limits: vec![],
            profiles: vec![],
            documentary_relations: vec![],
            roles: vec![],
            modes: vec![],
            sources: vec![],
            created_at: String::new(),
            updated_at: String::new(),
            extra: BTreeMap::new(),
        }
    }
}
impl Project {
    pub fn item(&self, id: &str) -> Option<&Item> {
        self.items.iter().find(|n| n.id == id)
    }
    pub fn item_mut(&mut self, id: &str) -> Option<&mut Item> {
        self.items.iter_mut().find(|n| n.id == id)
    }
    pub fn colors(&self) -> impl Iterator<Item = &Item> {
        self.items.iter().filter(|i| i.is_color())
    }
    pub fn is_source_locked(&self, id: &str) -> bool {
        self.items.iter().any(|i| i.id == id && i.source_locked)
    }
    pub fn capture_for_color(&self, id: &str) -> Option<(&SourceAsset, &ColorCapture)> {
        self.sources.iter().find_map(|source| {
            source
                .captures
                .iter()
                .find(|capture| capture.color_id.as_deref() == Some(id))
                .map(|capture| (source, capture))
        })
    }
    pub fn count(&self) -> usize {
        self.colors().count()
    }
    pub fn resolve_role(&self, role: &Role, mode: Option<&str>) -> Option<&Item> {
        let id = mode
            .and_then(|m| self.modes.iter().find(|x| x.id == m))
            .and_then(|m| m.overrides.get(&role.id))
            .unwrap_or(&role.color_id);
        self.item(id)
    }
    pub fn descendants(&self, ids: &[String]) -> BTreeSet<String> {
        let mut all: BTreeSet<String> = ids.iter().cloned().collect();
        for _ in 0..101 {
            let before = all.len();
            for i in &self.items {
                if i.parent_id.as_ref().is_some_and(|p| all.contains(p)) {
                    all.insert(i.id.clone());
                }
            }
            if all.len() == before {
                break;
            }
        }
        all
    }
    pub fn freeze_harmonies(&mut self) {
        let colors: BTreeMap<String, Rgb> = self.colors().map(|n| (n.id.clone(), n.rgb)).collect();
        for h in &mut self.harmonies {
            if !h.constraints.is_empty() {
                continue;
            }
            let root = h.anchor_id.as_ref().or_else(|| h.ids.first());
            if let Some(root) = root
                && let Some(base) = colors.get(root)
            {
                let base = color::hsv(*base);
                h.constraints = h
                    .ids
                    .iter()
                    .filter(|id| *id != root)
                    .filter_map(|id| {
                        colors.get(id).map(|rgb| {
                            let v = color::hsv(*rgb);
                            Constraint {
                                a: root.clone(),
                                b: id.clone(),
                                offset: [
                                    color::hue_delta(v[0], base[0]),
                                    v[1] - base[1],
                                    v[2] - base[2],
                                ],
                            }
                        })
                    })
                    .collect();
            }
        }
    }
    pub fn validate(&self) -> Result<()> {
        if self.schema_version > 4 {
            return Err("Arquivo de uma versão mais nova. Abra-o numa versão compatível; o original foi preservado.".into());
        }
        if self.id.is_empty()
            || self.name.len() > 4096
            || self.items.len() > 10000
            || self.links.len() > 20000
            || self.harmonies.len() > 2000
            || self.gradients.len() > 2000
            || self.limits.len() > 20000
            || self.sources.len() > 1000
        {
            return Err("Projeto excede os limites suportados.".into());
        }
        let ids: BTreeSet<_> = self.items.iter().map(|i| i.id.as_str()).collect();
        if ids.len() != self.items.len() {
            return Err("Há IDs de itens repetidos.".into());
        }
        let mut source_ids = BTreeSet::new();
        let mut capture_ids = BTreeSet::new();
        let mut linked_colors = BTreeSet::new();
        let mut embedded_bytes = 0usize;
        for source in &self.sources {
            if source.id.is_empty()
                || !source_ids.insert(source.id.as_str())
                || source.name.len() > 4096
                || source.path.len() > 32768
                || !matches!(source.storage.as_str(), "linked" | "embedded")
                || source.page_count == 0
                || source.captures.len() > 10000
                || source.preview_png.len() > 1024 * 1024
            {
                return Err("Fonte inválida ou fora dos limites suportados.".into());
            }
            if let Some(encoded) = &source.embedded_data {
                let decoded = base64::Engine::decode(
                    &base64::engine::general_purpose::STANDARD,
                    encoded.as_str(),
                )
                .map_err(|_| "Dados de fonte incorporada inválidos.")?;
                if decoded.len() > 64 * 1024 * 1024 {
                    return Err("Uma fonte incorporada excede 64 MiB.".into());
                }
                embedded_bytes = embedded_bytes.saturating_add(decoded.len());
            }
            if (source.storage == "embedded") != source.embedded_data.is_some() {
                return Err(
                    "O modo de armazenamento da fonte não corresponde aos seus dados.".into(),
                );
            }
            for capture in &source.captures {
                if capture.id.is_empty()
                    || !capture_ids.insert(capture.id.as_str())
                    || capture.name.len() > 4096
                    || capture.page >= source.page_count
                    || capture
                        .position
                        .iter()
                        .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
                    || !color::in_gamut(capture.value)
                    || capture.strokes.len() > 10000
                {
                    return Err("Captura inválida ou fora dos limites suportados.".into());
                }
                match &capture.method {
                    CaptureMethod::Area {
                        shape,
                        statistic,
                        radius,
                    } if !matches!(shape.as_str(), "circle" | "square")
                        || !matches!(statistic.as_str(), "mean" | "median")
                        || !(1..=128).contains(radius) =>
                    {
                        return Err("Parâmetros da área de captura inválidos.".into());
                    }
                    CaptureMethod::Brush { statistic }
                        if !matches!(statistic.as_str(), "mean" | "median") =>
                    {
                        return Err("Estatística da máscara inválida.".into());
                    }
                    _ => {}
                }
                for stroke in &capture.strokes {
                    if stroke.points.len() > 100000
                        || !stroke.radius.is_finite()
                        || !(0.5..=256.0).contains(&stroke.radius)
                        || stroke
                            .points
                            .iter()
                            .flatten()
                            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
                    {
                        return Err("Máscara de captura inválida ou grande demais.".into());
                    }
                }
                if let Some(id) = &capture.color_id {
                    let item = self
                        .item(id)
                        .filter(|i| i.is_color())
                        .ok_or("Captura referencia uma cor ausente.")?;
                    if !linked_colors.insert(id.as_str()) || !item.source_locked {
                        return Err(
                            "Vínculo de captura duplicado ou cor sem bloqueio de fonte.".into()
                        );
                    }
                }
            }
        }
        // Base64 expands binary assets by one third; reserve room for the rest of the JSON.
        if embedded_bytes > 180 * 1024 * 1024 {
            return Err("Fontes incorporadas excedem 180 MiB por projeto (limite ajustado ao tamanho JSON).".into());
        }
        if self
            .items
            .iter()
            .any(|item| item.source_locked && !linked_colors.contains(item.id.as_str()))
        {
            return Err("Há cor bloqueada por fonte sem uma captura vinculada.".into());
        }
        let mut profiles = BTreeSet::new();
        for p in &self.profiles {
            if p.id.is_empty() || !profiles.insert(&p.id) || p.bytes.len() > 4 * 1024 * 1024 {
                return Err("Perfil ICC inválido ou grande demais.".into());
            }
            craft_cms::Profile::parse(&p.bytes).map_err(|e| format!("Perfil ICC inválido: {e}"))?;
        }
        for i in &self.items {
            if i.id.is_empty() || !matches!(i.kind.as_str(), "color" | "folder") {
                return Err("Item de tipo desconhecido.".into());
            }
            let mut parent = i.parent_id.as_deref();
            let mut seen = BTreeSet::from([i.id.as_str()]);
            let mut depth = 0;
            while let Some(id) = parent {
                if !seen.insert(id) || depth >= 100 {
                    return Err("Ciclo ou excesso de níveis nas pastas.".into());
                }
                let node = self.item(id).ok_or("Pasta de origem ausente")?;
                if node.kind != "folder" {
                    return Err("O pai de um item precisa ser uma pasta.".into());
                }
                parent = node.parent_id.as_deref();
                depth += 1;
            }
            if i.is_color() {
                if !color::in_gamut(i.rgb) {
                    return Err("A prévia RGB contém valores inválidos.".into());
                }
                let n = i.native();
                let expected = if n.space == "cmyk" {
                    4
                } else if n.space == "gray" {
                    1
                } else {
                    3
                };
                if n.components.len() != expected || n.components.iter().any(|x| !x.is_finite()) {
                    return Err("Componentes nativos inválidos.".into());
                }
                if let Some(id) = &n.profile_id
                    && !profiles.contains(id)
                {
                    return Err("Uma cor referencia um perfil ICC ausente.".into());
                }
                if n.space != "lab" && n.components.iter().any(|x| *x < 0.0 || *x > 1.0) {
                    return Err("Componentes fora do intervalo nativo permitido.".into());
                }
                let profile = color::profile(&n, &self.profiles)?;
                let expected_space = match n.space.as_str() {
                    "srgb" | "display-p3" => craft_cms::ColorSpace::Rgb,
                    "cmyk" => craft_cms::ColorSpace::Cmyk,
                    "lab" => craft_cms::ColorSpace::Lab,
                    "gray" => craft_cms::ColorSpace::Gray,
                    _ => return Err("Modelo de cor desconhecido.".into()),
                };
                if profile.channels() != n.components.len() || profile.color_space != expected_space
                {
                    return Err("O perfil ICC e os componentes usam modelos diferentes.".into());
                }
                if n.space == "lab"
                    && (!(0.0..=100.0).contains(&n.components[0])
                        || n.components
                            .iter()
                            .skip(1)
                            .any(|v| !(-128.0..=127.0).contains(v)))
                {
                    return Err("Lab exige L* de 0 a 100 e a*/b* de -128 a 127.".into());
                }
            }
        }
        let is_color = |id: &str| self.item(id).is_some_and(Item::is_color);
        let mut relids = BTreeSet::new();
        for l in &self.links {
            if !relids.insert(&l.id)
                || l.a == l.b
                || !is_color(&l.a)
                || !is_color(&l.b)
                || l.channels.is_empty()
            {
                return Err("Vínculo inválido.".into());
            }
            for &c in &l.channels {
                let r = l.relations.get(&c.to_string()).ok_or("Canal sem relação")?;
                if c > 5
                    || !r.value.is_finite()
                    || !matches!(r.mode.as_str(), "ratio" | "offset")
                    || (r.mode == "ratio" && r.value.abs() < 1e-12)
                {
                    return Err("Parâmetros de vínculo inválidos.".into());
                }
            }
        }
        for h in &self.harmonies {
            if h.ids.len() < 2
                || h.ids.len() > 10000
                || h.ids.iter().any(|id| !is_color(id))
                || h.constraints.iter().any(|e| {
                    !is_color(&e.a) || !is_color(&e.b) || e.offset.iter().any(|x| !x.is_finite())
                })
            {
                return Err("Harmonia com referência inválida.".into());
            }
        }
        for g in &self.gradients {
            if !is_color(&g.a)
                || !is_color(&g.b)
                || g.a == g.b
                || g.ids.is_empty()
                || g.ids.len() > 10000
                || g.ids.iter().any(|id| !is_color(id))
                || !matches!(g.space.as_str(), "oklab" | "oklch" | "srgb")
            {
                return Err("Gradiente inválido.".into());
            }
        }
        for l in &self.limits {
            if !is_color(&l.a)
                || !is_color(&l.b)
                || l.a == l.b
                || !l.minimum.is_finite()
                || !(1.0..=21.0).contains(&l.minimum)
                || l.metric != "wcag22-srgb"
            {
                return Err("Limite de contraste inválido.".into());
            }
        }
        for d in &self.documentary_relations {
            if !ids.contains(d.from.as_str()) || !ids.contains(d.to.as_str()) {
                return Err("Relação documental com referência ausente.".into());
            }
        }
        let roleids: BTreeSet<_> = self.roles.iter().map(|r| r.id.as_str()).collect();
        if roleids.len() != self.roles.len()
            || self
                .roles
                .iter()
                .map(|r| r.name.trim())
                .collect::<BTreeSet<_>>()
                .len()
                != self.roles.len()
        {
            return Err("Papéis semânticos precisam ter IDs e nomes distintos.".into());
        }
        for role in &self.roles {
            if role.id.is_empty() || role.name.trim().is_empty() {
                return Err("Preencha o nome de cada papel.".into());
            }
            if !role.color_id.is_empty() && !is_color(&role.color_id) {
                return Err("Papel semântico referencia uma cor ausente.".into());
            }
        }
        let mut modeids = BTreeSet::new();
        let mut modenames = BTreeSet::new();
        for mode in &self.modes {
            if mode.id.is_empty()
                || mode.name.trim().is_empty()
                || !modeids.insert(&mode.id)
                || !modenames.insert(&mode.name)
            {
                return Err("Modos exigem IDs e nomes únicos, não vazios.".into());
            }
            for (role, col) in &mode.overrides {
                if !roleids.contains(role.as_str()) || (!col.is_empty() && !is_color(col)) {
                    return Err("Modo contém associação inválida.".into());
                }
            }
        }
        Ok(())
    }
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > 256 * 1024 * 1024 {
            return Err("Arquivo maior que 256 MiB.".into());
        }
        let mut p: Self =
            serde_json::from_slice(bytes).map_err(|e| format!("JSON de projeto inválido: {e}"))?;
        if p.schema_version > 4 {
            return Err("Este projeto exige uma versão mais nova do ColorCraft.".into());
        }
        p.validate()?;
        p.schema_version = 4;
        // Display caches from a file are not authoritative colour values.
        let previews: Vec<_> = p
            .items
            .iter()
            .filter(|i| i.is_color() && i.color.is_some())
            .map(|i| color::preview(&i.native(), &p.profiles).map(|rgb| (i.id.clone(), rgb)))
            .collect::<Result<_>>()?;
        for (id, rgb) in previews {
            if let Some(i) = p.item_mut(&id) {
                i.rgb = rgb.map(|v| v.clamp(0.0, 255.0));
            }
        }
        p.freeze_harmonies();
        Ok(p)
    }
    pub fn demo() -> Self {
        let mut p = Self {
            name: "Horizonte · estudo cromático".into(),
            description: "Uma paleta para explorar, conectar e documentar cores.".into(),
            ..Self::default()
        };
        for (name, rgb) in [
            ("Água clara", [65.0, 213.0, 230.0]),
            ("Violeta do horizonte", [146.0, 117.0, 239.0]),
            ("Pôr do sol", [245.0, 174.0, 91.0]),
            ("Texto principal", [31.0, 34.0, 42.0]),
            ("Papel", [246.0, 242.0, 231.0]),
        ] {
            p.items.push(Item {
                name: name.into(),
                css_name: color::css_name(name),
                rgb,
                ..Item::default()
            });
        }
        p
    }
}
