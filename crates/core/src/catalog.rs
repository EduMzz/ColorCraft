//! Compile-time catalog shared by editors and core operations. IDs are persistence keys.
//! New types register metadata, parameters and their command/generator here; see docs/ADDING-RELATIONS.md.
use crate::{
    Result,
    color::{self, Rgb},
    engine::Command,
};

#[derive(Clone, Copy, Debug)]
pub struct TypeInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
}
#[derive(Clone, Copy, Debug)]
pub struct NumberParameter {
    pub label: &'static str,
    pub min: f64,
    pub max: f64,
    pub default: f64,
    pub suffix: &'static str,
    pub suggestions: &'static [f64],
}
#[derive(Clone, Copy, Debug)]
pub struct Channel {
    pub index: usize,
    pub letter: char,
    pub name: &'static str,
    pub model: &'static str,
}
pub const CHANNELS: &[Channel] = &[
    Channel {
        index: 0,
        letter: 'R',
        name: "Vermelho",
        model: "RGB",
    },
    Channel {
        index: 1,
        letter: 'G',
        name: "Verde",
        model: "RGB",
    },
    Channel {
        index: 2,
        letter: 'B',
        name: "Azul",
        model: "RGB",
    },
    Channel {
        index: 3,
        letter: 'H',
        name: "Matiz",
        model: "HSV",
    },
    Channel {
        index: 4,
        letter: 'S',
        name: "Saturação",
        model: "HSV",
    },
    Channel {
        index: 5,
        letter: 'V',
        name: "Valor / brilho",
        model: "HSV",
    },
];

pub struct ParametricType {
    pub info: TypeInfo,
    pub channels: &'static [Channel],
    pub build: fn(Vec<String>, Vec<usize>) -> Result<Command>,
}
pub struct LimitType {
    pub info: TypeInfo,
    pub parameter: NumberParameter,
    pub current_label: &'static str,
    pub measure: fn(Rgb, Rgb) -> f64,
    pub explanation: &'static str,
    pub build: fn(String, String, f64) -> Result<Command>,
}
pub struct HarmonyType {
    pub info: TypeInfo,
    pub spread: Option<NumberParameter>,
    pub count: Option<NumberParameter>,
    pub generate: fn(Rgb, f64, usize) -> Vec<Rgb>,
}

pub const PARAMETRIC_TYPES: &[ParametricType] = &[ParametricType {
    info: TypeInfo {
        id: "channels",
        name: "Canais RGB / HSV",
        description: "Qualquer participante pode ser editado. Os canais selecionados preservam as proporções atuais; a matiz preserva a distância no círculo.",
        icon: "link",
    },
    channels: CHANNELS,
    build: |ids, channels| {
        if ids.len() < 2 || channels.is_empty() || channels.iter().any(|c| *c > 5) {
            return Err("Escolha pelo menos duas cores e um canal válido.".into());
        }
        Ok(Command::LinkColors { ids, channels })
    },
}];
pub const LIMIT_TYPES: &[LimitType] = &[LimitType {
    info: TypeInfo {
        id: "contrast",
        name: "Contraste",
        description: "Impõe uma razão mínima de contraste entre duas cores.",
        icon: "contrast",
    },
    parameter: NumberParameter {
        label: "Contraste mínimo",
        min: 1.0,
        max: 21.0,
        default: 4.5,
        suffix: ":1",
        suggestions: &[3.0, 4.5, 7.0],
    },
    current_label: "Contraste atual",
    measure: color::contrast,
    explanation: "WCAG 2.2 · cores opacas em sRGB. Uma cor pode variar até o limite; a outra não é ajustada para compensar.",
    build: |a, b, minimum| {
        if a.is_empty()
            || b.is_empty()
            || a == b
            || !minimum.is_finite()
            || !(1.0..=21.0).contains(&minimum)
        {
            return Err("Escolha duas cores diferentes e um contraste entre 1 e 21.".into());
        }
        Ok(Command::AddContrast { a, b, minimum })
    },
}];
const SPREAD: NumberParameter = NumberParameter {
    label: "Abertura",
    min: 5.0,
    max: 90.0,
    default: 30.0,
    suffix: "°",
    suggestions: &[15.0, 30.0, 45.0, 60.0],
};
const RECTANGLE: NumberParameter = NumberParameter {
    label: "Separação da tétrade",
    default: 45.0,
    suggestions: &[30.0, 45.0, 60.0, 90.0],
    ..SPREAD
};
const COUNT: NumberParameter = NumberParameter {
    label: "Quantidade",
    min: 2.0,
    max: 32.0,
    default: 5.0,
    suffix: "",
    suggestions: &[3.0, 5.0, 7.0, 9.0, 12.0],
};
const GEOMETRY_COUNT: NumberParameter = NumberParameter {
    label: "Quantidade de cores",
    ..COUNT
};
fn angles(base: Rgb, offsets: &[f64]) -> Vec<Rgb> {
    let [h, s, v] = color::hsv(base);
    offsets
        .iter()
        .map(|d| color::from_hsv([h + d, s, v]))
        .collect()
}
fn scale(base: Rgb, count: usize, shades: bool) -> Vec<Rgb> {
    let [h, s, v] = color::hsv(base);
    let count = count.clamp(2, 32);
    (0..count)
        .map(|i| {
            let t = i as f64 / (count - 1) as f64;
            color::from_hsv([
                h,
                if shades { s } else { s * (0.35 + 0.65 * t) },
                v * (0.25 + 0.75 * t),
            ])
        })
        .collect()
}
/// Places one hue at each vertex of a regular n-gon on the HSV hue circle.
/// One full revolution is 360 degrees, so adjacent vertices differ by 360/n.
fn geometry(base: Rgb, count: usize) -> Vec<Rgb> {
    let count = count.clamp(2, 32);
    let [h, s, v] = color::hsv(base);
    (0..count)
        .map(|i| color::from_hsv([h + i as f64 * 360.0 / count as f64, s, v]))
        .collect()
}
macro_rules! harmony {
    ($id:literal,$name:literal,$description:literal,$spread:expr,$count:expr,$generate:expr) => {
        HarmonyType {
            info: TypeInfo {
                id: $id,
                name: $name,
                description: $description,
                icon: "spark",
            },
            spread: $spread,
            count: $count,
            generate: $generate,
        }
    };
}
pub const HARMONY_TYPES: &[HarmonyType] = &[
    harmony!(
        "complementary",
        "Complementar",
        "Duas cores opostas na roda. Contraste direto e equilíbrio.",
        None,
        None,
        |b, _, _| angles(b, &[0.0, 180.0])
    ),
    harmony!(
        "split",
        "Complementar dividida",
        "A base e dois matizes próximos ao seu oposto.",
        None,
        None,
        |b, _, _| angles(b, &[0.0, 150.0, 210.0])
    ),
    harmony!(
        "analogous",
        "Análoga",
        "Matizes vizinhos, com abertura angular ajustável.",
        Some(SPREAD),
        None,
        |b, s, _| angles(b, &[-2.0 * s, -s, 0.0, s, 2.0 * s])
    ),
    harmony!(
        "triad",
        "Tríade",
        "Três matizes separados por 120°, com variedade equilibrada.",
        None,
        None,
        |b, _, _| angles(b, &[0.0, 120.0, 240.0])
    ),
    harmony!(
        "square",
        "Quadrado",
        "Quatro matizes em intervalos de 90°.",
        None,
        None,
        |b, _, _| angles(b, &[0.0, 90.0, 180.0, 270.0])
    ),
    harmony!(
        "geometry",
        "Geometria",
        "Distribui as cores igualmente pelos vértices de um polígono regular na roda de matizes.",
        None,
        Some(GEOMETRY_COUNT),
        |b, _, n| geometry(b, n)
    ),
    harmony!(
        "rectangle",
        "Tétrade retangular",
        "Dois pares complementares; ajuste a separação entre eles.",
        Some(RECTANGLE),
        None,
        |b, s, _| angles(b, &[0.0, s, 180.0, 180.0 + s])
    ),
    harmony!(
        "compound",
        "Composta",
        "Vizinhos da base combinados com cores próximas ao oposto.",
        Some(SPREAD),
        None,
        |b, s, _| angles(b, &[0.0, s, -s, 180.0 - s, 180.0 + s])
    ),
    harmony!(
        "shades",
        "Sombras",
        "Um matiz e saturação constantes, com valores de brilho diferentes.",
        None,
        Some(COUNT),
        |b, _, n| scale(b, n, true)
    ),
    harmony!(
        "mono",
        "Monocromática",
        "Um matiz, com variações de saturação e brilho.",
        None,
        Some(COUNT),
        |b, _, n| scale(b, n, false)
    ),
];
pub fn harmony(id: &str) -> Option<&'static HarmonyType> {
    HARMONY_TYPES.iter().find(|t| t.info.id == id)
}
pub fn parametric(id: &str) -> Option<&'static ParametricType> {
    PARAMETRIC_TYPES.iter().find(|t| t.info.id == id)
}
pub fn limit(id: &str) -> Option<&'static LimitType> {
    LIMIT_TYPES.iter().find(|t| t.info.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registered_types_have_unique_ids_and_valid_parameters_and_commands() {
        let ids: Vec<_> = PARAMETRIC_TYPES
            .iter()
            .map(|x| x.info.id)
            .chain(LIMIT_TYPES.iter().map(|x| x.info.id))
            .chain(HARMONY_TYPES.iter().map(|x| x.info.id))
            .collect();
        let unique: std::collections::BTreeSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len());
        for ty in HARMONY_TYPES {
            for p in [ty.spread, ty.count].into_iter().flatten() {
                assert!(p.min <= p.default && p.default <= p.max);
                assert!(p.suggestions.iter().all(|v| (p.min..=p.max).contains(v)));
            }
            let values = (ty.generate)([20.0, 100.0, 200.0], 30.0, 5);
            assert!(values.len() >= 2 && values.len() <= 32);
            assert!(values.iter().flatten().all(|v| v.is_finite()));
        }
        assert!((LIMIT_TYPES[0].build)("a".into(), "b".into(), 4.5).is_ok());
        assert!((LIMIT_TYPES[0].build)("a".into(), "a".into(), 4.5).is_err());
        assert!((PARAMETRIC_TYPES[0].build)(vec!["a".into(), "b".into()], vec![3]).is_ok());
    }

    #[test]
    fn geometry_harmony_uses_equal_regular_polygon_angles() {
        for count in 2..=32 {
            let hues: Vec<_> = geometry(color::from_hsv([347.0, 0.7, 0.8]), count)
                .into_iter()
                .map(|rgb| color::hsv(rgb)[0])
                .collect();
            assert_eq!(hues.len(), count);
            let expected = 360.0 / count as f64;
            for pair in hues.windows(2) {
                assert!((color::hue_delta(pair[1], pair[0]).abs() - expected).abs() < 1e-8);
            }
            assert!(hues.first().zip(hues.last()).is_some_and(|(first, last)| {
                (color::hue_delta(*first, *last).abs() - expected).abs() < 1e-8
            }));
        }
    }
}
