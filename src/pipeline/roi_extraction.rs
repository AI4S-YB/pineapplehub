use image::GrayImage;
use imageproc::{
    contours::{self, BorderType},
    geometry::{contour_area as geometry_contour_area, min_area_rect},
    point::Point,
};

use crate::error::Error;

use super::COIN_RADIUS_MM;

#[derive(Clone, Copy, Debug)]
pub(crate) struct RotatedRect {
    pub(crate) cx: f32,
    pub(crate) cy: f32,
    pub(crate) width: f32,     // Upright Width
    pub(crate) height: f32,    // Upright Height
    pub(crate) angle_rad: f32, // Rotation applied to image to make it upright
}

pub(crate) fn get_rotated_rect_info(points: &[Point<i32>]) -> RotatedRect {
    // We expect 4 points.
    if points.len() != 4 {
        return RotatedRect {
            cx: 0.0,
            cy: 0.0,
            width: 0.0,
            height: 0.0,
            angle_rad: 0.0,
        };
    }

    // Convert to float for simpler math
    let pts: Vec<(f32, f32)> = points.iter().map(|p| (p.x as f32, p.y as f32)).collect();

    // Calculate Edge Lengths
    // Edge 0: 0-1
    // Edge 1: 1-2
    let d0 = ((pts[1].0 - pts[0].0).powi(2) + (pts[1].1 - pts[0].1).powi(2)).sqrt();
    let d1 = ((pts[2].0 - pts[1].0).powi(2) + (pts[2].1 - pts[1].1).powi(2)).sqrt();

    let cx = (pts[0].0 + pts[1].0 + pts[2].0 + pts[3].0) / 4.0;
    let cy = (pts[0].1 + pts[1].1 + pts[2].1 + pts[3].1) / 4.0;

    // Identify Long Axis
    // Pineapple is usually Taller than Wide.
    // We want the Long Axis to be Vertical (Y).

    let (width, height, theta) = if d0 > d1 {
        // Edge 0 is Height
        // Angle of Edge 0
        let dx = pts[1].0 - pts[0].0;
        let dy = pts[1].1 - pts[0].1;
        let theta = dy.atan2(dx);
        (d1, d0, theta)
    } else {
        // Edge 1 is Height
        let dx = pts[2].0 - pts[1].0;
        let dy = pts[2].1 - pts[1].1;
        let theta = dy.atan2(dx);
        (d0, d1, theta)
    };

    // Calculate minimal rotation to vertical
    // We want to rotate such that the long axis becomes vertical.
    // This could be -PI/2 (Up) or PI/2 (Down).
    // We choose the rotation with smallest magnitude to avoid flipping the image upside down
    // if it is already mostly upright.

    let pi = std::f32::consts::PI;
    let normalize = |mut r: f32| {
        while r <= -pi {
            r += 2.0 * pi;
        }
        while r > pi {
            r -= 2.0 * pi;
        }
        r
    };

    let rot_up = normalize(-std::f32::consts::FRAC_PI_2 - theta);
    let rot_down = normalize(std::f32::consts::FRAC_PI_2 - theta);

    let angle = if rot_up.abs() < rot_down.abs() {
        rot_up
    } else {
        rot_down
    };

    RotatedRect {
        cx,
        cy,
        width,
        height,
        angle_rad: angle,
    }
}

pub(crate) fn extract_best_roi(
    smoothed: &GrayImage,
    px_per_mm: f32,
    contours: Vec<imageproc::contours::Contour<i32>>,
    fused: &GrayImage,
) -> Result<Option<RotatedRect>, Error> {
    // 2. Filter by Physical Area (Doc Step 2.3)
    // Area > 0.2 * Area_coin
    let coin_area_px = std::f32::consts::PI * (COIN_RADIUS_MM * px_per_mm).powi(2);
    let min_area = 0.2 * coin_area_px;

    let mut candidates = Vec::with_capacity(contours.len());
    for contour in contours {
        let area = geometry_contour_area(&contour.points).abs() as f32;
        if area > min_area {
            candidates.push((contour, area));
        }
    }

    // 3. Score Candidates by Texture Richness (edge density × area)
    // Skin side → bumpy fruitlet eyes → high local gradient magnitudes → high edge density.
    // Flesh side → smooth cut surface → low gradients → low edge density.
    // Coin → small area → penalized by area factor.
    let mut stats = Vec::with_capacity(candidates.len());

    for (i, (contour, area)) in candidates.iter().enumerate() {
        let rect = min_area_rect(&contour.points);
        let r_rect = get_rotated_rect_info(&rect);

        // Compute axis-aligned bounding box for this contour
        let (mut bx_min, mut by_min) = (i32::MAX, i32::MAX);
        let (mut bx_max, mut by_max) = (i32::MIN, i32::MIN);
        for pt in &contour.points {
            bx_min = bx_min.min(pt.x);
            by_min = by_min.min(pt.y);
            bx_max = bx_max.max(pt.x);
            by_max = by_max.max(pt.y);
        }

        // Clamp to image bounds
        let (img_w, img_h) = smoothed.dimensions();
        let bx0 = (bx_min.max(0) as u32).min(img_w.saturating_sub(1));
        let by0 = (by_min.max(0) as u32).min(img_h.saturating_sub(1));
        let bx1 = (bx_max.max(0) as u32).min(img_w.saturating_sub(1));
        let by1 = (by_max.max(0) as u32).min(img_h.saturating_sub(1));

        // Compute edge density: average |dI/dx| + |dI/dy| over non-background pixels
        let bg_threshold = 15u8; // pixels below this are considered black background
        let mut gradient_sum: f64 = 0.0;
        let mut pixel_count: u32 = 0;

        for y in by0..by1.min(img_h - 1) {
            for x in bx0..bx1.min(img_w - 1) {
                let p = smoothed.get_pixel(x, y).0[0];
                if p <= bg_threshold {
                    continue; // skip black background
                }
                let px_right = smoothed.get_pixel(x + 1, y).0[0];
                let py_down = smoothed.get_pixel(x, y + 1).0[0];
                let dx = (p as i16 - px_right as i16).unsigned_abs() as f64;
                let dy = (p as i16 - py_down as i16).unsigned_abs() as f64;
                gradient_sum += dx + dy;
                pixel_count += 1;
            }
        }

        let edge_density = if pixel_count > 0 {
            gradient_sum / pixel_count as f64
        } else {
            0.0
        };

        // Score = edge_density × sqrt(area)
        // sqrt(area) rather than area to avoid extreme dominance by size
        let score = edge_density as f32 * area.sqrt();

        log::info!(
            "[ROI Score] Candidate {}: area={:.0}, edge_density={:.2}, score={:.1}, rect={:?}",
            i,
            area,
            edge_density,
            score,
            r_rect
        );

        stats.push((i, r_rect, score));
    }

    // Sort by Score Descending
    stats.sort_by(|a, b| b.2.total_cmp(&a.2));

    if let Some(&(best_idx, _, score)) = stats.first() {
        // The best candidate identifies WHERE the peel-side fruit is, but its contour
        // (from the Otsu binary mask) may be fragmented. ALL fragment-AABB-based fixes
        // fail when the main fragment doesn't cover the full fruit extent.
        //
        // Solution: low-threshold the FULL smoothed grayscale image (fruit tissue ≈ 30+,
        // background ≈ 0-15) to get complete, unfragmented fruit silhouettes. Then match
        // the resulting contour to the scored target by centroid.
        let (best_contour, _) = &candidates[best_idx];

        // Compute best contour's centroid (to match against low-threshold contours)
        let n = best_contour.points.len() as i64;
        let (sx, sy) = best_contour.points.iter().fold((0i64, 0i64), |(sx, sy), pt| {
            (sx + pt.x as i64, sy + pt.y as i64)
        });
        let best_cx = if n > 0 { (sx / n) as i32 } else { 0 };
        let best_cy = if n > 0 { (sy / n) as i32 } else { 0 };

        // Low-threshold the FULL smoothed image — no crop, no AABB limitation.
        // No morphological closing: at threshold 25 the fruit is already one
        // connected component (smoothed fruitlet gaps ≈ 30-50, above 25), while
        // the background gap between objects (≈ 5-15) stays below 25, providing
        // natural separation without closing (which was bridging close objects).
        let low_binary = imageproc::contrast::threshold(
            smoothed,
            25,
            imageproc::contrast::ThresholdType::Binary,
        );

        // Find contours directly on the low-threshold image
        let low_contours = contours::find_contours::<i32>(&low_binary);

        // Find the outer contour whose AABB contains the best candidate's centroid
        // and has the largest AABB overlap with the best candidate
        let (bb_x_min, bb_y_min, bb_x_max, bb_y_max) = best_contour.points.iter().fold(
            (i32::MAX, i32::MAX, i32::MIN, i32::MIN),
            |(xn, yn, xx, yx), pt| (xn.min(pt.x), yn.min(pt.y), xx.max(pt.x), yx.max(pt.y)),
        );

        let matching = low_contours
            .iter()
            .filter(|c| c.border_type == BorderType::Outer)
            .filter(|c| {
                let (rx_min, ry_min, rx_max, ry_max) = c.points.iter().fold(
                    (i32::MAX, i32::MAX, i32::MIN, i32::MIN),
                    |(xn, yn, xx, yx), pt| {
                        (xn.min(pt.x), yn.min(pt.y), xx.max(pt.x), yx.max(pt.y))
                    },
                );
                best_cx >= rx_min && best_cx <= rx_max && best_cy >= ry_min && best_cy <= ry_max
            })
            .max_by_key(|c| {
                // Pick by AABB overlap area
                let (rx_min, ry_min, rx_max, ry_max) = c.points.iter().fold(
                    (i32::MAX, i32::MAX, i32::MIN, i32::MIN),
                    |(xn, yn, xx, yx), pt| {
                        (xn.min(pt.x), yn.min(pt.y), xx.max(pt.x), yx.max(pt.y))
                    },
                );
                let overlap_w = (bb_x_max.min(rx_max) - bb_x_min.max(rx_min)).max(0) as i64;
                let overlap_h = (bb_y_max.min(ry_max) - bb_y_min.max(ry_min)).max(0) as i64;
                overlap_w * overlap_h
            });

        let r_rect = if let Some(rc) = matching {
            let rect = min_area_rect(&rc.points);
            let merged = get_rotated_rect_info(&rect);
            log::info!(
                "[Step 5] Best ROI (full low-thresh, {} pts): score={:.2}, rect={:?}",
                rc.points.len(),
                score,
                merged
            );
            merged
        } else {
            // Fallback: use best contour's rect directly
            let rect = min_area_rect(&best_contour.points);
            let fallback = get_rotated_rect_info(&rect);
            log::info!(
                "[Step 5] Best ROI (fallback): score={:.2}, rect={:?}",
                score,
                fallback
            );
            fallback
        };

        Ok(Some(r_rect))
    } else {
        Err(Error::General("No valid ROI found".into()))
    }
}
