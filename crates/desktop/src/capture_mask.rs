//! Smooth mask display and stroke erasing. Sampling remains in the core, at source resolution.
use crate::app::SourceWorkbench;
use colorcraft_core::CaptureMethod;
use eframe::egui::{self, Color32, Pos2, Rect, Vec2};
use std::hash::{Hash, Hasher};

#[derive(Default)]
pub(crate) struct MaskOverlay {
    key: Option<u64>,
    texture: Option<egui::TextureHandle>,
}

fn distance(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.length_sq().max(f32::EPSILON)).clamp(0.0, 1.0);
    (p - a - ab * t).length()
}

fn segment_distance(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> f32 {
    let cross = |u: Vec2, v: Vec2| u.x * v.y - u.y * v.x;
    let denominator = cross(b - a, d - c);
    if denominator.abs() > f32::EPSILON {
        let t = cross(c - a, d - c) / denominator;
        let u = cross(c - a, b - a) / denominator;
        if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
            return 0.0;
        }
    }
    distance(a, c, d)
        .min(distance(b, c, d))
        .min(distance(c, a, b))
        .min(distance(d, a, b))
}

/// A fast pointer sweep is a segment, so it cannot skip a thin stroke between frames.
pub(crate) fn erase(wb: &mut SourceWorkbench, from: [f32; 2], to: [f32; 2], tolerance: f32) {
    let dimensions = Vec2::new(
        wb.image.width().saturating_sub(1) as f32,
        wb.image.height().saturating_sub(1) as f32,
    );
    let pixel = |p: [f32; 2]| Vec2::new(p[0], p[1]) * dimensions;
    let Some(cap) = wb.source.captures.get_mut(wb.selected) else {
        return;
    };
    if cap.page != wb.page || !matches!(cap.method, CaptureMethod::Brush { .. }) {
        return;
    }
    let before = cap.strokes.len();
    cap.strokes.retain(|stroke| {
        let radius = stroke.radius + tolerance;
        if let [point] = stroke.points.as_slice() {
            distance(pixel(*point), pixel(from), pixel(to)) > radius
        } else {
            !stroke.points.windows(2).any(|p| {
                segment_distance(pixel(from), pixel(to), pixel(p[0]), pixel(p[1])) <= radius
            })
        }
    });
    wb.dirty |= before != cap.strokes.len();
}

/// Maximum coverage, rather than alpha-over drawing, gives every stroke a uniform fill.
fn capsule(coverage: &mut [u8], size: [usize; 2], a: Vec2, b: Vec2, radius: f32) {
    let lo = a.min(b) - Vec2::splat(radius + 1.0);
    let hi = a.max(b) + Vec2::splat(radius + 1.0);
    let left = (lo.x.floor().max(0.0) as usize).min(size[0]);
    let top = (lo.y.floor().max(0.0) as usize).min(size[1]);
    let right = (hi.x.ceil().max(0.0) as usize + 1).min(size[0]);
    let bottom = (hi.y.ceil().max(0.0) as usize + 1).min(size[1]);
    for y in top..bottom {
        for x in left..right {
            let alpha = ((radius + 0.5 - distance(Vec2::new(x as f32, y as f32), a, b))
                .clamp(0.0, 1.0)
                * 255.0)
                .round() as u8;
            let i = y * size[0] + x;
            coverage[i] = coverage[i].max(alpha);
        }
    }
}

fn raster(wb: &SourceWorkbench) -> egui::ColorImage {
    let width = wb.image.width() as f32;
    let height = wb.image.height() as f32;
    let scale = (1024.0 / width.max(height)).min(1.0);
    let size = [
        (width * scale).ceil().max(1.0) as usize,
        (height * scale).ceil().max(1.0) as usize,
    ];
    let mut selected = vec![0; size[0] * size[1]];
    let mut others = vec![0; selected.len()];
    let dimensions = Vec2::new(
        size[0].saturating_sub(1) as f32,
        size[1].saturating_sub(1) as f32,
    );
    let point = |p: [f32; 2]| Vec2::new(p[0], p[1]) * dimensions;
    for (index, cap) in wb
        .source
        .captures
        .iter()
        .enumerate()
        .filter(|(_, c)| c.page == wb.page && matches!(c.method, CaptureMethod::Brush { .. }))
    {
        let coverage = if index == wb.selected {
            &mut selected
        } else {
            &mut others
        };
        for stroke in &cap.strokes {
            let radius = (stroke.radius * scale).max(0.6);
            if let [p] = stroke.points.as_slice() {
                capsule(coverage, size, point(*p), point(*p), radius);
            }
            for pair in stroke.points.windows(2) {
                capsule(coverage, size, point(pair[0]), point(pair[1]), radius);
            }
        }
    }
    let mut pixels = vec![Color32::TRANSPARENT; selected.len()];
    for y in 0..size[1] {
        for x in 0..size[0] {
            let i = y * size[0] + x;
            let (coverage, rgb, opacity) = if selected[i] > 0 {
                (&selected, [255, 205, 25], 76_u16)
            } else {
                (&others, [210, 230, 240], 45_u16)
            };
            if coverage[i] == 0 {
                continue;
            }
            let edge = [
                (x.saturating_sub(1), y),
                ((x + 1).min(size[0] - 1), y),
                (x, y.saturating_sub(1)),
                (x, (y + 1).min(size[1] - 1)),
            ]
            .iter()
            .any(|&(xx, yy)| coverage[yy * size[0] + xx] < 128);
            let alpha = ((coverage[i] as u16 * if edge { 190 } else { opacity }) / 255) as u8;
            pixels[i] = Color32::from_rgba_unmultiplied(rgb[0], rgb[1], rgb[2], alpha);
        }
    }
    egui::ColorImage::new(size, pixels)
}

pub(crate) fn paint(ui: &egui::Ui, wb: &mut SourceWorkbench, painter: &egui::Painter, rect: Rect) {
    let mut hash = std::hash::DefaultHasher::new();
    wb.page.hash(&mut hash);
    wb.selected.hash(&mut hash);
    wb.image.dimensions().hash(&mut hash);
    for cap in wb
        .source
        .captures
        .iter()
        .filter(|c| c.page == wb.page && matches!(c.method, CaptureMethod::Brush { .. }))
    {
        cap.id.hash(&mut hash);
        cap.strokes.len().hash(&mut hash);
        for stroke in &cap.strokes {
            stroke.radius.to_bits().hash(&mut hash);
            for p in &stroke.points {
                p[0].to_bits().hash(&mut hash);
                p[1].to_bits().hash(&mut hash);
            }
        }
    }
    let key = hash.finish();
    if wb.mask_overlay.key != Some(key) {
        let image = raster(wb);
        if let Some(texture) = &mut wb.mask_overlay.texture {
            texture.set(image, egui::TextureOptions::LINEAR);
        } else {
            wb.mask_overlay.texture = Some(ui.ctx().load_texture(
                "capture-mask-overlay",
                image,
                egui::TextureOptions::LINEAR,
            ));
        }
        wb.mask_overlay.key = Some(key);
    }
    if let Some(texture) = &wb.mask_overlay.texture {
        painter.image(
            texture.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coverage_at_joints_and_crossings_does_not_accumulate() {
        let mut mask = vec![0; 100 * 100];
        capsule(
            &mut mask,
            [100, 100],
            Vec2::new(10.0, 50.0),
            Vec2::new(50.0, 50.0),
            6.0,
        );
        capsule(
            &mut mask,
            [100, 100],
            Vec2::new(50.0, 50.0),
            Vec2::new(90.0, 50.0),
            6.0,
        );
        assert_eq!(mask[50 * 100 + 30], mask[50 * 100 + 50]);
        assert_eq!(mask[50 * 100 + 50], 255);
        capsule(
            &mut mask,
            [100, 100],
            Vec2::new(50.0, 10.0),
            Vec2::new(50.0, 90.0),
            6.0,
        );
        assert_eq!(mask[50 * 100 + 50], 255);
        assert!(mask[56 * 100 + 40] > 0 && mask[56 * 100 + 40] < 255);
    }
    #[test]
    fn crossing_and_single_point_hit_geometry() {
        assert_eq!(
            segment_distance(
                Vec2::new(0.0, 5.0),
                Vec2::new(10.0, 5.0),
                Vec2::new(5.0, 0.0),
                Vec2::new(5.0, 10.0)
            ),
            0.0
        );
        assert_eq!(
            distance(Vec2::new(5.0, 5.0), Vec2::ZERO, Vec2::new(10.0, 0.0)),
            5.0
        );
    }
}
