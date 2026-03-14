# ROI Extraction: Known Issues & Improvement Directions

> Source: edge cases discovered during `fix/algo` branch development, 2026-03-14

## Background

The ROI extraction pipeline in [`roi_extraction.rs`](../src/pipeline/roi_extraction.rs) uses a **two-stage design**:

1. **Locate** — Score candidate contours (from the Otsu binary mask) by texture richness (`edge_density × √area`) to identify the peel-side fruit.
2. **Bound** — Low-threshold the full smoothed grayscale image (`threshold = 25`) to obtain a complete, unfragmented fruit silhouette. Match back to the scored target by centroid, then compute `min_area_rect`.

This solves the contour fragmentation problem where Otsu binarization splits the peel surface into disconnected fragments along dark inter-fruitlet gaps.

## Known Edge Case

### Peel and cross-section merging at low threshold

**Symptom**: On certain images where the peel-side fruit and the cross-section are placed very close together (< 5 mm), the smoothed grayscale pixels in the gap are raised close to threshold 25. After thresholding, both objects become one connected component.

**Impact**: `find_contours` returns a single large contour spanning both objects → `min_area_rect` computes a ROI covering both → downstream unwrap fails.

**Status**: Observed on 1 image so far.

## Potential Improvements

### Approach A: Adaptive threshold

**Idea**: Replace the fixed threshold (25) with one derived from the image's own brightness distribution, via `otsu_level(smoothed) / 3`.

```rust
let otsu = imageproc::contrast::otsu_level(smoothed);
let low_thresh_val = (otsu / 3).max(15).min(50);
let low_binary = threshold(smoothed, low_thresh_val, Binary);
```

**Rationale**:

| Image condition | Otsu | Resulting threshold | Effect |
|-----------------|------|---------------------|--------|
| Typical         | ~120 | 40                  | High enough to separate close objects, low enough to cover fruit |
| Low light       | ~80  | ~27                 | More permissive for dim fruit tissue |
| Bright          | ~150 | 50 (capped)         | Avoids overly high threshold that fragments |

**Risk**: Threshold 40 may exclude the darkest fruitlet gap pixels on some images, causing contour concavities. Requires validation across the full image set.

### Approach B: Area validation with fallback

**Idea**: After matching the low-threshold contour, validate its area against the original candidate's area. If it's disproportionately large (> 2×), it likely spans multiple objects — fall back to the original contour.

```rust
let matched_area = geometry_contour_area(&rc.points).abs() as f32;
let (_, candidate_area) = &candidates[best_idx];
if matched_area > candidate_area * 2.0 {
    log::warn!(
        "[Step 5] Low-thresh contour too large ({:.0} vs {:.0}), falling back",
        matched_area, candidate_area
    );
    let rect = min_area_rect(&best_contour.points);
    get_rotated_rect_info(&rect)
} else {
    let rect = min_area_rect(&rc.points);
    get_rotated_rect_info(&rect)
}
```

**Pros**:
- Does not change the threshold; preserves fragmentation fix for all currently-working images
- Fallback to original contour is safe (equivalent to pre-fix behavior)

**Cons**:
- The 2× multiplier needs empirical tuning
- Fallback means the fragmentation fix doesn't apply to that particular image (but the original code couldn't fix it either)

### Approach C: Combine A + B

Use adaptive threshold (A) first. If the resulting contour still triggers the area anomaly check, fall back (B). Double safety net.

## Recommended Priority

1. **Approach B** (area validation) — simplest to implement, lowest risk, fully backward-compatible
2. **Approach A** (adaptive threshold) — requires more test images; can be layered on top of B
