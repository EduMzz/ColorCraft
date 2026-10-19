//! ColorCraft integration v1. MIT OR Apache-2.0. No dependency on either editor.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub type Result<T> = std::result::Result<T, String>;
pub const MAX_JSON: u64 = 256 * 1024 * 1024;
pub const MAX_REQUEST: u64 = 32 * 1024 * 1024;
pub const VERSION: u32 = 1;
pub fn uid() -> String {
    uuid::Uuid::new_v4().to_string()
}
pub fn digest(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}
/// Exact native components and ICC bytes, independent of application-local IDs.
pub fn equivalent(a: &NativeColor, ap: &[Profile], b: &NativeColor, bp: &[Profile]) -> bool {
    let same_model = a.space == b.space
        || (matches!(a.space.as_str(), "srgb" | "display-p3")
            && matches!(b.space.as_str(), "srgb" | "display-p3"));
    if !same_model || a.components != b.components {
        return false;
    }
    fn profile(c: &NativeColor, ps: &[Profile]) -> Option<Vec<u8>> {
        if let Some(id) = &c.profile_id {
            return ps.iter().find(|p| &p.id == id).map(|p| p.bytes.clone());
        }
        use craft_cms::Builtin;
        Some(
            match c.space.as_str() {
                "srgb" => Builtin::Srgb,
                "display-p3" => Builtin::DisplayP3,
                "lab" => Builtin::LabD50,
                "cmyk" => Builtin::CoatedCmyk,
                "gray" => Builtin::GrayGamma22,
                _ => return None,
            }
            .profile()
            .to_bytes()
            .as_ref()
            .clone(),
        )
    }
    profile(a, ap).is_some_and(|x| profile(b, bp).is_some_and(|y| x == y))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeColor {
    pub space: String,
    pub components: Vec<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
}
impl NativeColor {
    pub fn validate(&self) -> Result<()> {
        let channels = match self.space.as_str() {
            "srgb" | "display-p3" | "lab" => 3,
            "cmyk" => 4,
            "gray" => 1,
            _ => return Err("Unsupported color space".into()),
        };
        if self.components.len() != channels || self.components.iter().any(|n| !n.is_finite()) {
            return Err("Invalid color components".into());
        }
        for (i, n) in self.components.iter().enumerate() {
            let (lo, hi) = if self.space == "lab" {
                if i == 0 {
                    (0.0, 100.0)
                } else {
                    (-128.0, 127.0)
                }
            } else {
                (0.0, 1.0)
            };
            if *n < lo || *n > hi {
                return Err("Color component outside its authoring range".into());
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub bytes: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub color: Option<NativeColor>,
    #[serde(default)]
    pub preview_srgb: Option<[f64; 3]>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub format: String,
    pub schema_version: u32,
    pub palette_id: String,
    pub revision: u64,
    pub name: String,
    pub items: Vec<Item>,
    #[serde(default)]
    pub profiles: Vec<Profile>,
    #[serde(default, flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}
impl Snapshot {
    pub fn validate(&self) -> Result<()> {
        if self.format != "colorcraft-palette" || self.schema_version != 1 {
            return Err("Unsupported ColorCraft palette version".into());
        }
        if self.palette_id.is_empty() || self.name.len() > 16384 || self.items.len() > 10000 {
            return Err("Invalid ColorCraft palette".into());
        }
        let mut ids = BTreeSet::new();
        for item in &self.items {
            if item.id.is_empty() || !ids.insert(&item.id) || item.name.len() > 16384 {
                return Err("Duplicate or invalid palette item".into());
            }
            match item.kind.as_str() {
                "color" => {
                    item.color
                        .as_ref()
                        .ok_or("Color has no native value")?
                        .validate()?;
                    if item
                        .preview_srgb
                        .is_none_or(|v| v.iter().any(|x| !x.is_finite() || *x < 0.0 || *x > 255.0))
                    {
                        return Err("Invalid sRGB preview".into());
                    }
                }
                "folder" => {}
                _ => return Err("Unsupported palette item".into()),
            }
        }
        let map: BTreeMap<_, _> = self.items.iter().map(|i| (&i.id, i)).collect();
        for item in &self.items {
            let mut parent = item.parent_id.as_ref();
            let mut seen = BTreeSet::new();
            while let Some(id) = parent {
                if !seen.insert(id) || id == &item.id || seen.len() > 128 {
                    return Err("Palette folder cycle or excessive depth".into());
                }
                let p = map.get(id).ok_or("Missing palette folder")?;
                if p.kind != "folder" {
                    return Err("Palette parent is not a folder".into());
                }
                parent = p.parent_id.as_ref();
            }
        }
        let mut profiles = BTreeSet::new();
        let mut bytes = 0usize;
        for p in &self.profiles {
            bytes = bytes.saturating_add(p.bytes.len());
            if !profiles.insert(&p.id)
                || p.id.is_empty()
                || p.bytes.len() > 16 * 1024 * 1024
                || bytes > 64 * 1024 * 1024
            {
                return Err("Invalid or excessive ICC data".into());
            }
        }
        for i in &self.items {
            if let Some(id) = i.color.as_ref().and_then(|c| c.profile_id.as_ref())
                && !profiles.contains(id)
            {
                return Err("Missing source ICC profile".into());
            }
        }
        Ok(())
    }
    pub fn item(&self, id: &str) -> Option<&Item> {
        self.items.iter().find(|i| i.id == id)
    }
    pub fn path(&self, id: &str) -> String {
        let mut names = Vec::new();
        let mut next = self.item(id);
        let mut seen = BTreeSet::new();
        while let Some(i) = next {
            if !seen.insert(&i.id) {
                break;
            }
            names.push(i.name.as_str());
            next = i.parent_id.as_deref().and_then(|p| self.item(p));
        }
        names.reverse();
        names.join(" / ")
    }
}
/// Convert the authoring value, never its clipped preview, to portable Lab D50.
pub fn lab(value: &NativeColor, profiles: &[Profile]) -> Result<[f64; 3]> {
    use craft_cms::{Builtin, Intent, Profile as CmsProfile, Transform};
    value.validate()?;
    let src = if let Some(id) = &value.profile_id {
        CmsProfile::parse(
            &profiles
                .iter()
                .find(|p| &p.id == id)
                .ok_or("Missing ICC profile")?
                .bytes,
        )
        .map_err(|e| e.to_string())?
    } else {
        match value.space.as_str() {
            "srgb" => Builtin::Srgb,
            "display-p3" => Builtin::DisplayP3,
            "lab" => Builtin::LabD50,
            "cmyk" => Builtin::CoatedCmyk,
            "gray" => Builtin::GrayGamma22,
            _ => return Err("Unsupported color space".into()),
        }
        .profile()
        .clone()
    };
    let transform = Transform::new(
        &src,
        Builtin::LabD50.profile(),
        Intent::RelativeColorimetric,
        true,
    )
    .map_err(|e| e.to_string())?;
    let input: Vec<f32> = value
        .components
        .iter()
        .enumerate()
        .map(|(i, v)| {
            if value.space == "lab" {
                if i == 0 {
                    (v / 100.0) as f32
                } else {
                    ((v + 128.0) / 255.0) as f32
                }
            } else {
                *v as f32
            }
        })
        .collect();
    let mut out = [0f32; 3];
    transform
        .eval(&input, &mut out)
        .map_err(|e| e.to_string())?;
    let result = [
        f64::from(out[0]) * 100.0,
        f64::from(out[1]) * 255.0 - 128.0,
        f64::from(out[2]) * 255.0 - 128.0,
    ];
    if result.iter().any(|x| !x.is_finite()) {
        return Err("Non-finite ICC conversion".into());
    }
    Ok(result)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryEntry {
    pub palette_id: String,
    pub revision: u64,
    pub name: String,
    pub snapshot: String,
    pub source_path: String,
    #[serde(default)]
    pub colors: usize,
    #[serde(default)]
    pub preview: Vec<[f64; 3]>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub schema_version: u32,
    pub palettes: Vec<LibraryEntry>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewColor {
    pub name: String,
    pub color: NativeColor,
    #[serde(default)]
    pub profiles: Vec<Profile>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub schema_version: u32,
    pub request_id: String,
    pub operation: String,
    pub palette_id: String,
    #[serde(default)]
    pub folder_id: Option<String>,
    #[serde(default)]
    pub color_id: Option<String>,
    #[serde(default)]
    pub expected_state: Option<String>,
    #[serde(default)]
    pub colors: Vec<NewColor>,
    #[serde(default)]
    pub profiles: Vec<Profile>,
    #[serde(default)]
    pub language: String,
}
impl Request {
    pub fn new(operation: &str, palette: &str) -> Self {
        Self {
            schema_version: VERSION,
            request_id: uid(),
            operation: operation.into(),
            palette_id: palette.into(),
            folder_id: None,
            color_id: None,
            expected_state: None,
            colors: vec![],
            profiles: vec![],
            language: String::new(),
        }
    }
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != VERSION
            || uuid::Uuid::parse_str(&self.request_id).is_err()
            || self.palette_id.is_empty()
            || !["inspect", "append", "open", "publish"].contains(&self.operation.as_str())
            || self.colors.len() > 1000
        {
            return Err("Invalid bridge request".into());
        }
        let mut total_profiles = 0usize;
        for p in self
            .profiles
            .iter()
            .chain(self.colors.iter().flat_map(|c| &c.profiles))
        {
            total_profiles = total_profiles.saturating_add(p.bytes.len());
            if p.bytes.len() > 16 * 1024 * 1024 || total_profiles > 64 * 1024 * 1024 {
                return Err("ICC data exceeds the bridge limit".into());
            }
        }
        for c in &self.colors {
            c.color.validate()?;
            if c.name.trim().is_empty() || c.name.len() > 16384 {
                return Err("Invalid color name".into());
            }
            for p in &c.profiles {
                if p.bytes.len() > 16 * 1024 * 1024 {
                    return Err("ICC profile too large".into());
                }
            }
        }
        if self.operation == "append" && (self.colors.is_empty() || self.expected_state.is_none()) {
            return Err("Append needs colors and a confirmed destination state".into());
        }
        Ok(())
    }
    pub fn from_uri(uri: &str) -> Result<Self> {
        let url = url::Url::parse(uri).map_err(|e| e.to_string())?;
        if url.scheme() != "colorcraft"
            || url.host_str() != Some("open")
            || !["", "/"].contains(&url.path())
            || url.fragment().is_some()
        {
            return Err("Unsupported ColorCraft URI".into());
        }
        let mut palette = None;
        let mut color = None;
        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "palette" if palette.is_none() => palette = Some(value.into_owned()),
                "color" if color.is_none() => color = Some(value.into_owned()),
                _ => return Err("Invalid ColorCraft URI parameters".into()),
            }
        }
        let mut r = Self::new("open", &palette.ok_or("Missing palette ID")?);
        r.color_id = color;
        r.validate()?;
        Ok(r)
    }
}
pub fn open_uri(palette: &str, color: Option<&str>) -> String {
    let mut result = format!(
        "colorcraft://open?palette={}",
        url::form_urlencoded::byte_serialize(palette.as_bytes()).collect::<String>()
    );
    if let Some(id) = color {
        result.push_str("&color=");
        result.push_str(&url::form_urlencoded::byte_serialize(id.as_bytes()).collect::<String>());
    }
    result
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub schema_version: u32,
    pub request_id: String,
    pub ok: bool,
    pub status: String,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub dirty: bool,
    #[serde(default)]
    pub snapshot: Option<Snapshot>,
    #[serde(default)]
    pub color_ids: Vec<String>,
}
impl Response {
    pub fn new(r: &Request, status: &str) -> Self {
        Self {
            schema_version: VERSION,
            request_id: r.request_id.clone(),
            ok: true,
            status: status.into(),
            error: None,
            state: None,
            dirty: false,
            snapshot: None,
            color_ids: vec![],
        }
    }
    pub fn error(r: &Request, status: &str, error: String) -> Self {
        let mut v = Self::new(r, status);
        v.ok = false;
        v.error = Some(error);
        v
    }
}
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod native;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn uri_is_strict_and_roundtrips() {
        let r = Request::from_uri(&open_uri("p é", Some("c&1"))).unwrap();
        assert_eq!(r.palette_id, "p é");
        assert_eq!(r.color_id.as_deref(), Some("c&1"));
        for u in [
            "https://open?palette=p",
            "colorcraft://other?palette=p",
            "colorcraft://open?path=C:/x",
            "colorcraft://open?palette=p&palette=q",
        ] {
            assert!(Request::from_uri(u).is_err());
        }
    }
    #[test]
    fn conversions_are_native_and_bounded() {
        for (space, components) in [
            ("srgb", vec![1.0, 0.0, 0.0]),
            ("display-p3", vec![1.0, 0.0, 0.0]),
            ("lab", vec![60.0, 10.0, -20.0]),
            ("cmyk", vec![0.1, 0.2, 0.3, 0.1]),
            ("gray", vec![0.4]),
        ] {
            let c = NativeColor {
                space: space.into(),
                components,
                profile_id: None,
            };
            assert!(lab(&c, &[]).unwrap().iter().all(|x| x.is_finite()));
        }
        let bad = NativeColor {
            space: "srgb".into(),
            components: vec![f64::NAN, 0.0, 0.0],
            profile_id: None,
        };
        assert!(lab(&bad, &[]).is_err());
        let missing = NativeColor {
            space: "srgb".into(),
            components: vec![0.5; 3],
            profile_id: Some("missing".into()),
        };
        assert!(lab(&missing, &[]).is_err());
    }
    #[test]
    fn exact_matches_compare_profile_contents_across_application_ids() {
        let a = NativeColor {
            space: "display-p3".into(),
            components: vec![0.8, 0.2, 0.4],
            profile_id: None,
        };
        let b = NativeColor {
            space: "srgb".into(),
            components: a.components.clone(),
            profile_id: Some("document-rgb".into()),
        };
        let profile = Profile {
            id: "document-rgb".into(),
            name: "Custom P3".into(),
            bytes: craft_cms::Builtin::DisplayP3
                .profile()
                .to_bytes()
                .as_ref()
                .clone(),
        };
        assert!(equivalent(&a, &[], &b, &[profile]));
        assert!(!equivalent(
            &a,
            &[],
            &NativeColor {
                profile_id: None,
                ..b
            },
            &[]
        ));
        let lab_value = NativeColor {
            space: "lab".into(),
            components: vec![63.0, 17.0, -26.0],
            profile_id: None,
        };
        for (actual, expected) in lab(&lab_value, &[])
            .unwrap()
            .iter()
            .zip(&lab_value.components)
        {
            assert!((actual - expected).abs() < 0.001);
        }
    }
}
