//! Colour mathematics. sRGB transfer: IEC 61966-2-1 / W3C CSS Color 4.
//! OKLab matrices: https://bottosson.github.io/posts/oklab/ . No competitor code.
use crate::{IccProfile, NativeColor, Result};
use craft_cms::{Builtin, Intent, Profile, Transform};

pub type Rgb = [f64; 3];
pub fn wrap(h: f64) -> f64 {
    h.rem_euclid(360.0)
}
pub fn hue_delta(a: f64, b: f64) -> f64 {
    (a - b + 180.0).rem_euclid(360.0) - 180.0
}
pub fn hex(rgb: Rgb) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        rgb[0].clamp(0.0, 255.0).round() as u8,
        rgb[1].clamp(0.0, 255.0).round() as u8,
        rgb[2].clamp(0.0, 255.0).round() as u8
    )
}
pub fn from_hex(s: &str) -> Result<Rgb> {
    let s = s.trim().trim_start_matches('#');
    let s = if s.len() == 3 && s.is_ascii() {
        s.chars().flat_map(|c| [c, c]).collect::<String>()
    } else {
        s.to_string()
    };
    if s.len() != 6 || !s.is_ascii() {
        return Err("Use uma cor HEX de 3 ou 6 dígitos.".into());
    }
    let mut rgb = [0.0; 3];
    for (i, out) in rgb.iter_mut().enumerate() {
        *out = u8::from_str_radix(s.get(i * 2..i * 2 + 2).ok_or("HEX inválido")?, 16)
            .map_err(|_| "HEX inválido")? as f64;
    }
    Ok(rgb)
}
pub fn parse(s: &str) -> Result<Rgb> {
    let s = s.trim();
    if !s.contains('(') {
        return from_hex(s);
    }
    let (mode, rest) = s.split_once('(').ok_or("Código de cor inválido")?;
    let rest = rest.strip_suffix(')').ok_or("Código de cor incompleto")?;
    let tokens: Vec<_> = rest
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
        .collect();
    if tokens.len() != 3 {
        return Err(
            "Use três componentes. Para CMYK, use o editor nativo com perfil explícito.".into(),
        );
    }
    let values: Vec<f64> = tokens
        .iter()
        .map(|s| {
            s.trim_end_matches('%')
                .parse::<f64>()
                .map_err(|_| "Componente inválido".to_string())
        })
        .collect::<Result<_>>()?;
    if values.iter().any(|x| !x.is_finite()) {
        return Err("Valor de cor não finito.".into());
    }
    let v = [values[0], values[1], values[2]];
    if matches!(mode.trim().to_lowercase().as_str(), "hsl" | "hsv")
        && v.iter().skip(1).any(|x| !(0.0..=100.0).contains(x))
    {
        return Err("Saturação e luminosidade/valor devem estar entre 0 e 100%.".into());
    }
    let rgb = match mode.trim().to_lowercase().as_str() {
        "rgb" => std::array::from_fn(|i| {
            if tokens[i].ends_with('%') {
                v[i] * 2.55
            } else {
                v[i]
            }
        }),
        "hsl" => from_hsl([v[0], v[1] / 100.0, v[2] / 100.0]),
        "hsv" => from_hsv([v[0], v[1] / 100.0, v[2] / 100.0]),
        "oklch" => raw_oklch([
            if tokens[0].ends_with('%') {
                v[0] / 100.0
            } else {
                v[0]
            },
            v[1],
            v[2],
        ]),
        _ => return Err("Use HEX, rgb(), hsl(), hsv() ou oklch().".into()),
    };
    if !in_gamut(rgb) {
        return Err("Esta cor excede sRGB. Use o editor nativo ou confirme um ajuste de gamut pelo editor OKLCH.".into());
    }
    Ok(rgb)
}
pub fn hsv(rgb: Rgb) -> [f64; 3] {
    let [r, g, b] = rgb.map(|x| x / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d.abs() < 1e-15 {
        0.0
    } else if max == r {
        60.0 * ((g - b) / d).rem_euclid(6.0)
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    [h, if max == 0.0 { 0.0 } else { d / max }, max]
}
pub fn from_hsv([h, s, v]: [f64; 3]) -> Rgb {
    let h = wrap(h);
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let p = if h < 60.0 {
        [c, x, 0.0]
    } else if h < 120.0 {
        [x, c, 0.0]
    } else if h < 180.0 {
        [0.0, c, x]
    } else if h < 240.0 {
        [0.0, x, c]
    } else if h < 300.0 {
        [x, 0.0, c]
    } else {
        [c, 0.0, x]
    };
    p.map(|z| (z + m) * 255.0)
}
pub fn hsl(rgb: Rgb) -> [f64; 3] {
    let [h, s, v] = hsv(rgb);
    let l = v * (1.0 - s / 2.0);
    [
        h,
        if l <= 0.0 || l >= 1.0 {
            0.0
        } else {
            (v - l) / l.min(1.0 - l)
        },
        l,
    ]
}
pub fn from_hsl([h, s, l]: [f64; 3]) -> Rgb {
    let v = l + s * l.min(1.0 - l);
    from_hsv([h, if v == 0.0 { 0.0 } else { 2.0 * (1.0 - l / v) }, v])
}
pub fn linear(x: f64) -> f64 {
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}
pub fn gamma(x: f64) -> f64 {
    if x <= 0.0031308 {
        12.92 * x
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    }
}
pub fn luminance(rgb: Rgb) -> f64 {
    let [r, g, b] = rgb.map(|x| linear(x / 255.0));
    0.2126 * r + 0.7152 * g + 0.0722 * b
}
pub fn contrast(a: Rgb, b: Rgb) -> f64 {
    let x = luminance(a);
    let y = luminance(b);
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}
pub fn oklab(rgb: Rgb) -> [f64; 3] {
    let [r, g, b] = rgb.map(|x| linear(x / 255.0));
    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
    [
        0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s,
    ]
}
pub fn oklch(rgb: Rgb) -> [f64; 3] {
    let [l, a, b] = oklab(rgb);
    [l, a.hypot(b), wrap(b.atan2(a).to_degrees())]
}
pub fn raw_oklch([ll, c, h]: [f64; 3]) -> Rgb {
    let a = c * h.to_radians().cos();
    let b = c * h.to_radians().sin();
    let l = (ll + 0.3963377774 * a + 0.2158037573 * b).powi(3);
    let m = (ll - 0.1055613458 * a - 0.0638541728 * b).powi(3);
    let s = (ll - 0.0894841775 * a - 1.291485548 * b).powi(3);
    [
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s,
    ]
    .map(|x| gamma(x) * 255.0)
}
pub fn in_gamut(rgb: Rgb) -> bool {
    rgb.iter()
        .all(|x| x.is_finite() && *x >= -0.00001 && *x <= 255.00001)
}
/// Explicit chroma reduction policy used only by recipes requesting this mapping.
pub fn map_chroma(lch: [f64; 3]) -> Rgb {
    let rgb = raw_oklch(lch);
    if in_gamut(rgb) {
        return rgb.map(|x| x.clamp(0.0, 255.0));
    }
    let mut lo = 0.0;
    let mut hi = lch[1];
    for _ in 0..30 {
        let mid = (lo + hi) / 2.0;
        if in_gamut(raw_oklch([lch[0], mid, lch[2]])) {
            lo = mid
        } else {
            hi = mid
        }
    }
    raw_oklch([lch[0], lo, lch[2]]).map(|x| x.clamp(0.0, 255.0))
}
pub fn interpolate(a: Rgb, b: Rgb, t: f64, space: &str) -> Rgb {
    if space == "srgb" {
        return std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t);
    }
    let aa = oklch(a);
    let bb = oklch(b);
    if space == "oklch" {
        return map_chroma([
            aa[0] + (bb[0] - aa[0]) * t,
            aa[1] + (bb[1] - aa[1]) * t,
            aa[2] + hue_delta(bb[2], aa[2]) * t,
        ]);
    }
    let x = oklab(a);
    let y = oklab(b);
    let l = x[0] + (y[0] - x[0]) * t;
    let ac = x[1] + (y[1] - x[1]) * t;
    let bc = x[2] + (y[2] - x[2]) * t;
    map_chroma([l, ac.hypot(bc), wrap(bc.atan2(ac).to_degrees())])
}
pub fn harmony_name(kind: &str) -> &str {
    crate::catalog::harmony(kind)
        .map(|t| t.info.name)
        .unwrap_or(kind)
}
pub fn harmony(base: Rgb, kind: &str, spread: f64, count: usize) -> Vec<Rgb> {
    // Keep the historical fallback for unknown persisted recipe IDs.
    crate::catalog::harmony(kind)
        .or_else(|| crate::catalog::harmony("mono"))
        .map(|t| (t.generate)(base, spread, count))
        .unwrap_or_default()
}
pub fn channel(rgb: Rgb, c: usize) -> f64 {
    if c < 3 {
        rgb.get(c).copied().unwrap_or(0.0)
    } else {
        hsv(rgb).get(c - 3).copied().unwrap_or(0.0)
    }
}
pub fn channel_distance(a: f64, b: f64, c: usize) -> f64 {
    if c == 3 {
        hue_delta(a, b).abs()
    } else {
        (a - b).abs()
    }
}

pub fn profile(value: &NativeColor, profiles: &[IccProfile]) -> Result<Profile> {
    if let Some(id) = &value.profile_id {
        let record = profiles
            .iter()
            .find(|p| &p.id == id)
            .ok_or("Perfil ICC ausente. Carregue o perfil original antes de converter.")?;
        return Profile::parse(&record.bytes).map_err(|e| format!("Perfil ICC inválido: {e}"));
    }
    let builtin = match value.space.as_str() {
        "srgb" => Builtin::Srgb,
        "display-p3" => Builtin::DisplayP3,
        "lab" => Builtin::LabD50,
        "cmyk" => Builtin::CoatedCmyk,
        "gray" => Builtin::GrayGamma22,
        _ => return Err("Espaço de cor não suportado.".into()),
    };
    Ok(builtin.profile().clone())
}
pub fn converted(
    value: &NativeColor,
    target: &NativeColor,
    profiles: &[IccProfile],
) -> Result<NativeColor> {
    let src = profile(value, profiles)?;
    let dst = profile(target, profiles)?;
    let transform = Transform::new(&src, &dst, Intent::RelativeColorimetric, true)
        .map_err(|e| e.to_string())?;
    let input: Vec<f32> = if value.space == "lab" {
        vec![
            (value.components.first().copied().unwrap_or(0.0) / 100.0) as f32,
            ((value.components.get(1).copied().unwrap_or(0.0) + 128.0) / 255.0) as f32,
            ((value.components.get(2).copied().unwrap_or(0.0) + 128.0) / 255.0) as f32,
        ]
    } else {
        value.components.iter().map(|x| *x as f32).collect()
    };
    let mut output = vec![0.0f32; dst.channels()];
    transform
        .eval(&input, &mut output)
        .map_err(|e| e.to_string())?;
    let mut components: Vec<f64> = output.into_iter().map(f64::from).collect();
    if target.space == "lab" {
        if let Some(x) = components.get_mut(0) {
            *x *= 100.0
        }
        for x in components.iter_mut().skip(1) {
            *x = *x * 255.0 - 128.0
        }
    }
    Ok(NativeColor {
        components,
        ..target.clone()
    })
}
pub fn preview(value: &NativeColor, profiles: &[IccProfile]) -> Result<Rgb> {
    if value.is_srgb() {
        return Ok([
            value.components.first().copied().unwrap_or(0.0) * 255.0,
            value.components.get(1).copied().unwrap_or(0.0) * 255.0,
            value.components.get(2).copied().unwrap_or(0.0) * 255.0,
        ]);
    }
    let target = NativeColor::srgb([0.0; 3]);
    let out = converted(value, &target, profiles)?;
    Ok([
        out.components.first().copied().unwrap_or(0.0) * 255.0,
        out.components.get(1).copied().unwrap_or(0.0) * 255.0,
        out.components.get(2).copied().unwrap_or(0.0) * 255.0,
    ])
}
pub fn css_name(s: &str) -> String {
    let mut out = String::new();
    for ch in s.to_lowercase().chars() {
        let ch = match ch {
            'á' | 'à' | 'â' | 'ã' => 'a',
            'é' | 'ê' => 'e',
            'í' => 'i',
            'ó' | 'ô' | 'õ' => 'o',
            'ú' | 'ü' => 'u',
            'ç' => 'c',
            x => x,
        };
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            out.push(ch)
        } else if !out.ends_with('-') {
            out.push('-')
        }
    }
    let out = out.trim_matches('-');
    if out.is_empty() {
        "cor".into()
    } else if out.starts_with(|c: char| c.is_ascii_digit()) {
        format!("cor-{out}")
    } else {
        out.into()
    }
}
