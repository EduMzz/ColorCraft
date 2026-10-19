use crate::{
    Result,
    color::Rgb,
    model::{CaptureMethod, ColorCapture},
};

/// Sample a rendered RGBA surface. Coordinates and brush paths are normalized,
/// so a capture is independent of viewport zoom and pan.
pub fn sample(width: u32, height: u32, rgba: &[u8], capture: &ColorCapture) -> Result<Rgb> {
    if width == 0
        || height == 0
        || width as usize * height as usize > 100_000_000
        || rgba.len() != width as usize * height as usize * 4
    {
        return Err("A imagem não pode ser amostrada com essas dimensões.".into());
    }
    let cx = (capture.position[0].clamp(0.0, 1.0) * width.saturating_sub(1) as f32).round() as i32;
    let cy = (capture.position[1].clamp(0.0, 1.0) * height.saturating_sub(1) as f32).round() as i32;
    if matches!(capture.method, CaptureMethod::Pixel) {
        if cx < 0 || cy < 0 || cx >= width as i32 || cy >= height as i32 {
            return Err("A área selecionada não contém pixels visíveis.".into());
        }
        let i = (cy as usize * width as usize + cx as usize) * 4;
        if rgba[i + 3] == 0 {
            return Err("A área selecionada não contém pixels visíveis.".into());
        }
        return Ok([rgba[i] as f64, rgba[i + 1] as f64, rgba[i + 2] as f64]);
    }
    let mut pixels: Vec<[f64; 4]> = Vec::new();
    match &capture.method {
        CaptureMethod::Pixel => push_pixel(&mut pixels, rgba, width, height, cx, cy),
        CaptureMethod::Area {
            shape,
            statistic: _,
            radius,
        } => {
            let r = (*radius).clamp(1, 128) as i32;
            for y in cy - r..=cy + r {
                for x in cx - r..=cx + r {
                    if shape == "circle" && (x - cx) * (x - cx) + (y - cy) * (y - cy) > r * r {
                        continue;
                    }
                    push_pixel(&mut pixels, rgba, width, height, x, y);
                }
            }
        }
        CaptureMethod::Brush { .. } => {
            if capture.strokes.is_empty() {
                return Err("Pinte uma máscara antes de capturar sua média.".into());
            }
            let fw = width as f64;
            let fh = height as f64;
            let strokes: Vec<(f64, Vec<[f64; 2]>)> = capture
                .strokes
                .iter()
                .map(|stroke| {
                    (
                        stroke.radius as f64,
                        stroke
                            .points
                            .iter()
                            .map(|p| [p[0] as f64 * (fw - 1.0), p[1] as f64 * (fh - 1.0)])
                            .collect(),
                    )
                })
                .collect();
            for y in 0..height as i32 {
                for x in 0..width as i32 {
                    if strokes.iter().any(|(radius, points)| {
                        if points.len() == 1 {
                            distance2([x as f64, y as f64], points[0]) <= radius * radius
                        } else {
                            points.windows(2).any(|pair| {
                                distance_segment2([x as f64, y as f64], pair[0], pair[1])
                                    <= radius * radius
                            })
                        }
                    }) {
                        push_pixel(&mut pixels, rgba, width, height, x, y);
                    }
                }
            }
        }
    }
    if pixels.is_empty() {
        return Err("A área selecionada não contém pixels visíveis.".into());
    }
    let statistic = match &capture.method {
        CaptureMethod::Area { statistic, .. } | CaptureMethod::Brush { statistic } => {
            statistic.as_str()
        }
        CaptureMethod::Pixel => "mean",
    };
    if statistic == "median" {
        median_oklab(&pixels)
    } else {
        mean_linear(&pixels)
    }
}

fn push_pixel(out: &mut Vec<[f64; 4]>, rgba: &[u8], width: u32, height: u32, x: i32, y: i32) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return;
    }
    let i = (y as usize * width as usize + x as usize) * 4;
    let alpha = rgba[i + 3] as f64 / 255.0;
    if alpha > 0.0 {
        out.push([
            rgba[i] as f64 / 255.0,
            rgba[i + 1] as f64 / 255.0,
            rgba[i + 2] as f64 / 255.0,
            alpha,
        ]);
    }
}
fn distance2(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)
}
fn distance_segment2(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let len = dx * dx + dy * dy;
    if len <= f64::EPSILON {
        return distance2(p, a);
    }
    let t = (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / len).clamp(0.0, 1.0);
    distance2(p, [a[0] + t * dx, a[1] + t * dy])
}
fn to_linear(v: f64) -> f64 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}
fn to_srgb(v: f64) -> f64 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}
fn mean_linear(pixels: &[[f64; 4]]) -> Result<Rgb> {
    let mut sum = [0.0; 3];
    let mut alpha = 0.0;
    for p in pixels {
        alpha += p[3];
        for c in 0..3 {
            sum[c] += to_linear(p[c]) * p[3];
        }
    }
    if alpha <= 0.0 {
        return Err("A área não contém pixels visíveis.".into());
    }
    Ok(std::array::from_fn(|c| to_srgb(sum[c] / alpha) * 255.0))
}
fn linear_to_lab(rgb: [f64; 3]) -> [f64; 3] {
    let r = to_linear(rgb[0]);
    let g = to_linear(rgb[1]);
    let b = to_linear(rgb[2]);
    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
    [
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
}
fn lab_to_srgb(lab: [f64; 3]) -> Rgb {
    let l = (lab[0] + 0.3963377774 * lab[1] + 0.2158037573 * lab[2]).powi(3);
    let m = (lab[0] - 0.1055613458 * lab[1] - 0.0638541728 * lab[2]).powi(3);
    let s = (lab[0] - 0.0894841775 * lab[1] - 1.2914855480 * lab[2]).powi(3);
    [
        to_srgb(4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s),
        to_srgb(-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s),
        to_srgb(-0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s),
    ]
    .map(|x| x * 255.0)
}
fn median_oklab(pixels: &[[f64; 4]]) -> Result<Rgb> {
    let labs: Vec<_> = pixels
        .iter()
        .map(|p| (linear_to_lab([p[0], p[1], p[2]]), p[3]))
        .collect();
    let mut lab = [0.0; 3];
    for c in 0..3 {
        let mut values: Vec<_> = labs.iter().map(|(v, w)| (v[c], *w)).collect();
        values.sort_by(|a, b| a.0.total_cmp(&b.0));
        let total: f64 = values.iter().map(|(_, w)| w).sum();
        let mut cumulative = 0.0;
        for (value, weight) in values {
            cumulative += weight;
            if cumulative >= total * 0.5 {
                lab[c] = value;
                break;
            }
        }
    }
    Ok(lab_to_srgb(lab))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use crate::model::{CaptureMethod, ColorCapture};
    fn cap(method: CaptureMethod) -> ColorCapture {
        ColorCapture {
            method,
            position: [0.5, 0.5],
            ..Default::default()
        }
    }
    #[test]
    fn pixel_samples_exact_center() {
        let got = sample(
            2,
            2,
            &[0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 51, 102, 153, 255],
            &cap(CaptureMethod::Pixel),
        )
        .unwrap();
        assert_eq!(got, [51.0, 102.0, 153.0]);
    }
    #[test]
    fn area_mean_weights_alpha_and_uses_linear_light() {
        let got = sample(
            2,
            1,
            &[0, 0, 0, 255, 255, 255, 255, 0],
            &ColorCapture {
                position: [0.0, 0.0],
                method: CaptureMethod::Area {
                    shape: "square".into(),
                    statistic: "mean".into(),
                    radius: 2,
                },
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(got, [0.0, 0.0, 0.0]);
    }
    #[test]
    fn brush_mask_samples_only_marked_pixels() {
        let c = ColorCapture {
            method: CaptureMethod::Brush {
                statistic: "mean".into(),
            },
            strokes: vec![crate::model::CaptureStroke {
                points: vec![[0.0, 0.0]],
                radius: 0.5,
            }],
            ..Default::default()
        };
        let got = sample(2, 1, &[255, 0, 0, 255, 0, 0, 255, 255], &c).unwrap();
        assert!((got[0] - 255.0).abs() < 1e-9);
        assert_eq!([got[1], got[2]], [0.0, 0.0]);
    }
    #[test]
    fn transparent_area_has_no_result() {
        assert!(sample(1, 1, &[10, 20, 30, 0], &cap(CaptureMethod::Pixel)).is_err());
    }
}
