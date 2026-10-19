use crate::{color, *};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Format {
    Palette,
    Project,
    Css,
    Json,
    Tokens,
    Gpl,
    Simple,
    Csv,
    Svg,
}
impl Format {
    pub const ALL: [Self; 9] = [
        Self::Palette,
        Self::Project,
        Self::Css,
        Self::Json,
        Self::Tokens,
        Self::Gpl,
        Self::Simple,
        Self::Csv,
        Self::Svg,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Palette => "Paleta ColorCraft",
            Self::Project => "Projeto editável ColorCraft",
            Self::Css => "CSS · variáveis",
            Self::Json => "JSON · paleta resolvida",
            Self::Tokens => "Design Tokens · DTCG 2025.10",
            Self::Gpl => "GIMP Palette · GPL",
            Self::Simple => "Simple Color Palette",
            Self::Csv => "CSV · tabela de cores",
            Self::Svg => "SVG · folha de amostras",
        }
    }
    pub fn extension(self) -> &'static str {
        match self {
            Self::Palette => "colorcraft-palette",
            Self::Project => "colorcraft",
            Self::Css => "css",
            Self::Json | Self::Tokens => "json",
            Self::Gpl => "gpl",
            Self::Simple => "color-palette",
            Self::Csv => "csv",
            Self::Svg => "svg",
        }
    }
}
pub fn palette(p: &Project, appendix: bool) -> Result<Value> {
    p.validate()?;
    let items: Vec<Value> = p
        .items
        .iter()
        .map(|i| {
            let mut v = serde_json::to_value(i).map_err(|e| e.to_string())?;
            if let Some(m) = v.as_object_mut() {
                m.remove("locked");
                m.remove("directLocked");
                m.remove("sourceLocked");
                m.remove("rgb");
                if i.is_color() {
                    m.insert("color".into(), json!(i.native()));
                    m.insert("previewSrgb".into(), json!(i.rgb));
                }
            }
            Ok(v)
        })
        .collect::<Result<_>>()?;
    let mut value = json!({"format":"colorcraft-palette","schemaVersion":1,"paletteId":p.id,"name":p.name,"description":p.description,"revision":p.revision,"createdAt":p.created_at,"updatedAt":p.updated_at,"previewSpace":"srgb","items":items,"profiles":p.profiles,"documentaryRelations":p.documentary_relations,"roles":p.roles,"modes":p.modes,"extensions":p.extra});
    if appendix {
        value["technicalAppendix"] = json!({"informationalOnly":true,"mathVersion":1,"links":p.links,"harmonies":p.harmonies,"gradients":p.gradients,"limits":p.limits});
    }
    Ok(value)
}
pub fn from_palette(bytes: &[u8]) -> Result<Project> {
    let v: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if v.get("format").and_then(Value::as_str) != Some("colorcraft-palette")
        || v.get("schemaVersion").and_then(Value::as_u64) != Some(1)
    {
        return Err("Versão de paleta não suportada.".into());
    }
    let raw = v
        .get("items")
        .and_then(Value::as_array)
        .ok_or("Paleta sem itens")?;
    let mut items = vec![];
    for item in raw {
        let mut item = item.clone();
        if let Some(rgb) = item.get("previewSrgb").cloned() {
            item["rgb"] = rgb;
        }
        if let Some(obj) = item.as_object_mut() {
            obj.remove("previewSrgb");
        }
        let item: Item = serde_json::from_value(item).map_err(|e| e.to_string())?;
        items.push(item);
    }
    let mut p = Project {
        name: v
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("Paleta importada")
            .into(),
        description: v
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("")
            .into(),
        items,
        ..Project::default()
    };
    p.profiles = serde_json::from_value(v.get("profiles").cloned().unwrap_or(json!([])))
        .map_err(|e| e.to_string())?;
    p.roles = serde_json::from_value(v.get("roles").cloned().unwrap_or(json!([])))
        .map_err(|e| e.to_string())?;
    p.modes = serde_json::from_value(v.get("modes").cloned().unwrap_or(json!([])))
        .map_err(|e| e.to_string())?;
    p.documentary_relations =
        serde_json::from_value(v.get("documentaryRelations").cloned().unwrap_or(json!([])))
            .map_err(|e| e.to_string())?;
    p.extra = serde_json::from_value(v.get("extensions").cloned().unwrap_or(json!({})))
        .map_err(|e| e.to_string())?;
    if let Some(id) = v.get("paletteId") {
        p.extra.insert("sourcePaletteId".into(), id.clone());
    }
    p.validate()?;
    let previews = p
        .colors()
        .map(|i| color::preview(&i.native(), &p.profiles).map(|rgb| (i.id.clone(), rgb)))
        .collect::<Result<Vec<_>>>()?;
    for (id, rgb) in previews {
        if let Some(item) = p.item_mut(&id) {
            item.rgb = rgb.map(|v| v.clamp(0.0, 255.0));
        }
    }
    Ok(p)
}
fn standard_token_color(p: &Project, native: &NativeColor) -> bool {
    if !matches!(native.space.as_str(), "srgb" | "display-p3" | "lab") {
        return false;
    }
    let Some(id) = &native.profile_id else {
        return true;
    };
    let builtin = NativeColor {
        profile_id: None,
        ..native.clone()
    };
    color::profile(&builtin, &[]).is_ok_and(|profile| {
        p.profiles.iter().any(|stored| {
            &stored.id == id && stored.bytes.as_slice() == profile.to_bytes().as_slice()
        })
    })
}
pub fn warnings(p: &Project, f: Format) -> Vec<String> {
    if matches!(f, Format::Palette | Format::Project | Format::Json) {
        return vec![];
    }
    let mut messages =
        vec!["Esta saída contém cores resolvidas; não guarda as regras técnicas de edição.".into()];
    if matches!(f, Format::Gpl | Format::Simple | Format::Csv | Format::Svg) {
        messages.push(
            "Pastas, IDs, papéis, modos e metadados completos não são preservados neste formato."
                .into(),
        );
    }
    if f == Format::Gpl {
        messages.push("GPL usa apenas sRGB de 8 bits. Valores serão convertidos quando necessário e arredondados; perfis ICC não acompanham o arquivo.".into());
    }
    let converts =
        |i: &Item| !i.is_srgb() && (f != Format::Tokens || !standard_token_color(p, &i.native()));
    if p.colors().any(converts) {
        messages.push("Há cores nativas fora de sRGB. Esta saída usa conversão colorimétrica para sRGB; a origem permanece no projeto.".into());
    }
    if p.colors().any(|i| {
        (f != Format::Tokens || !standard_token_color(p, &i.native()))
            && color::preview(&i.native(), &p.profiles).is_ok_and(|rgb| !color::in_gamut(rgb))
    }) && f != Format::Simple
    {
        messages.push("Há cores fora do gamut sRGB. A prévia/saída sRGB limita os canais; confira o resultado antes de exportar.".into());
    }
    if f == Format::Simple {
        messages.push("Simple Color Palette escreve sRGB linear com precisão de 0,00001, sem as estruturas adicionais do ColorCraft.".into());
    }
    messages
}
fn pretty(v: &Value) -> Result<Vec<u8>> {
    serde_json::to_vec_pretty(v).map_err(|e| e.to_string())
}
fn comment(s: &str) -> String {
    s.replace("*/", "* /").replace(['\n', '\r'], " ")
}
fn escaped(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
pub fn names(p: &Project) -> BTreeMap<String, String> {
    let mut seen = BTreeSet::new();
    p.colors()
        .map(|i| {
            let base = color::css_name(if i.css_name.is_empty() {
                &i.name
            } else {
                &i.css_name
            });
            let mut name = base.clone();
            let mut n = 2;
            while seen.contains(&name) {
                name = format!("{base}-{n}");
                n += 1;
            }
            seen.insert(name.clone());
            (i.id.clone(), name)
        })
        .collect()
}
pub fn encode(p: &Project, f: Format, appendix: bool) -> Result<Vec<u8>> {
    p.validate()?;
    if f == Format::Project {
        return serde_json::to_vec_pretty(p).map_err(|e| e.to_string());
    }
    if matches!(f, Format::Palette | Format::Json) {
        return pretty(&palette(p, appendix)?);
    }
    let names = names(p);
    let text = match f {
        Format::Css => {
            let mut lines = vec![
                format!("/* ColorCraft · {} */", comment(&p.name)),
                ":root {".into(),
            ];
            for i in &p.items {
                if i.kind == "folder" {
                    lines.push(format!("  /* Pasta: {} */", comment(&i.label())))
                } else {
                    let rgb =
                        color::preview(&i.native(), &p.profiles)?.map(|x| x.clamp(0.0, 255.0));
                    lines.push(format!(
                        "  /* {} · {} */",
                        comment(&i.label()),
                        comment(&i.usage)
                    ));
                    lines.push(format!(
                        "  --{}: rgb({:.6} {:.6} {:.6});",
                        names.get(&i.id).ok_or("Nome ausente")?,
                        rgb[0],
                        rgb[1],
                        rgb[2]
                    ));
                }
            }
            for r in &p.roles {
                if let Some(target) = names.get(&r.color_id) {
                    lines.push(format!(
                        "  --{}: var(--{});",
                        color::css_name(&format!("papel-{}", r.name)),
                        target
                    ));
                }
            }
            lines.push("}".into());
            for mode in &p.modes {
                lines.push(format!(
                    "[data-colorcraft-mode=\"{}\"] {{",
                    color::css_name(&mode.name)
                ));
                for role in &p.roles {
                    if let Some(id) = mode.overrides.get(&role.id) {
                        let property = color::css_name(&format!("papel-{}", role.name));
                        if let Some(target) = names.get(id) {
                            lines.push(format!("  --{property}: var(--{target});"));
                        } else if id.is_empty() {
                            lines.push(format!("  --{property}: initial;"));
                        }
                    }
                }
                lines.push("}".into());
            }
            lines.join("\n")
        }
        Format::Gpl => {
            let mut lines = vec![
                "GIMP Palette".into(),
                format!("Name: {}", p.name.replace(['\r', '\n'], " ")),
                "Columns: 8".into(),
                "# Exportado pelo ColorCraft; sRGB, 8 bits".into(),
            ];
            for i in p.colors() {
                let rgb = color::preview(&i.native(), &p.profiles)?
                    .map(|x| x.clamp(0.0, 255.0).round() as u8);
                lines.push(format!(
                    "{:3} {:3} {:3}\t{}",
                    rgb[0],
                    rgb[1],
                    rgb[2],
                    i.label().replace(['\n', '\r'], " ")
                ));
            }
            lines.join("\n")
        }
        Format::Simple => {
            if p.count() == 0 {
                return Err("Simple Color Palette exige pelo menos uma cor.".into());
            }
            let mut colors = vec![];
            for i in p.colors() {
                let rgb = color::preview(&i.native(), &p.profiles)?;
                colors.push(json!({"name":i.label(),"components":rgb.map(|x|(color::linear(x/255.0)*100000.0).round_ties_even()/100000.0)}));
            }
            return pretty(&json!({"name":p.name,"colors":colors}));
        }
        Format::Tokens => {
            let mut tokens = serde_json::Map::new();
            for i in p.colors() {
                let native = i.native();
                let (space, components) = if standard_token_color(p, &native) {
                    (native.space.clone(), native.components.clone())
                } else {
                    (
                        "srgb".into(),
                        color::preview(&native, &p.profiles)?
                            .map(|x| x.clamp(0.0, 255.0) / 255.0)
                            .to_vec(),
                    )
                };
                let token = json!({"$type":"color","$value":{"colorSpace":space,"components":components,"alpha":1},"$description":i.description,"$extensions":{"colorcraft":{"id":i.id,"parentId":i.parent_id,"usage":i.usage,"tags":i.tags,"custom":i.custom,"native":native}}});
                tokens.insert(names.get(&i.id).cloned().ok_or("Nome ausente")?, token);
            }
            let mut root = json!({"colors":tokens,"$extensions":{"colorcraft":{"paletteId":p.id,"profiles":p.profiles,"roles":p.roles,"modes":p.modes,"documentaryRelations":p.documentary_relations,"folders":p.items.iter().filter(|i|i.kind=="folder").collect::<Vec<_>>()}}});
            let mut roles = serde_json::Map::new();
            for role in &p.roles {
                if let Some(target) = names.get(&role.color_id) {
                    roles.insert(color::css_name(&role.name),json!({"$type":"color","$value":format!("{{colors.{target}}}"),"$description":role.description}));
                }
            }
            root["roles"] = json!(roles);
            return pretty(&root);
        }
        Format::Csv => {
            let q = |s: &str| format!("\"{}\"", s.replace('"', "\"\""));
            let mut lines=vec!["id;nome;pasta;espaco_nativo;componentes_nativos;perfil;hex_preview_srgb;uso;descricao".into()];
            for i in p.colors() {
                let n = i.native();
                lines.push(
                    [
                        q(&i.id),
                        q(&i.label()),
                        q(&i.parent_id.clone().unwrap_or_default()),
                        q(&n.space),
                        q(&format!("{:?}", n.components)),
                        q(n.profile_id.as_deref().unwrap_or("builtin")),
                        q(&color::hex(i.rgb)),
                        q(&i.usage),
                        q(&i.description),
                    ]
                    .join(";"),
                );
            }
            lines.join("\r\n")
        }
        Format::Svg => {
            let count = p.count();
            let columns = 5;
            let rows = count.div_ceil(columns);
            let mut s = format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1000\" height=\"{}\" viewBox=\"0 0 1000 {}\"><title>{}</title><rect width=\"100%\" height=\"100%\" fill=\"#181a20\"/>",
                rows * 180 + 80,
                rows * 180 + 80,
                escaped(&p.name)
            );
            s.push_str(&format!("<text x=\"24\" y=\"40\" fill=\"white\" font-family=\"sans-serif\" font-size=\"24\">{}</text>",escaped(&p.name)));
            for (i, item) in p.colors().enumerate() {
                let x = (i % columns) * 196 + 14;
                let y = (i / columns) * 180 + 65;
                s.push_str(&format!("<rect x=\"{x}\" y=\"{y}\" width=\"184\" height=\"110\" rx=\"8\" fill=\"{}\"/><text x=\"{}\" y=\"{}\" fill=\"white\" font-family=\"sans-serif\" font-size=\"13\">{}</text><text x=\"{}\" y=\"{}\" fill=\"#b5b8c0\" font-family=\"monospace\" font-size=\"12\">{}</text>",color::hex(item.rgb),x+4,y+130,escaped(&item.label()),x+4,y+150,color::hex(item.rgb)));
            }
            s.push_str("</svg>");
            s
        }
        _ => return Err("Formato indisponível.".into()),
    };
    Ok(text.into_bytes())
}
