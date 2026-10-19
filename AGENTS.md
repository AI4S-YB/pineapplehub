# Project Guidance

## Collection And Annotation

- Agronomy graduate students collect images and annotate them. Assume no
  computer-vision background; programming, data conversion, dataset splitting,
  model training, and deployment belong to the development team.
- Give students one repeatable workflow in plain Chinese. Use a single selected
  photo per fruit for both training and evaluation collection. Do not require
  multi-view matching, manually entered fruit IDs, or student-selected dataset
  splits. Developers retain identity/provenance internally and keep retakes,
  duplicates, crops, and augmentations out of different splits.
- Ask students to mark visible regions and recorded measurements, not technical
  training eligibility or estimated occlusion percentages. Use one escalation
  action for all unresolved cases: refer to the project reviewer.
- Save each fruitlet as a separate region. Distinguish incomplete but identifiable
  fruitlets from regions whose boundaries or instance count cannot be determined.
  Uncertain regions are neither fruitlet instances nor confirmed background.
- Avoid wasting usable data, but favor straightforward exclusion after review
  over substantial custom code to salvage ambiguous supervision. Additional
  careful collection/annotation is acceptable. Exclusion from a training set
  does not authorize deleting original files.
- Issue #21 is the students' official collection/annotation guide. Keep model
  research, training architecture, and development milestones out of its body:
  https://github.com/AI4S-YB/pineapplehub/issues/21
- Real tasks, annotations, and drafts now exist. Preserve original files, region
  IDs, measurements, and draft identity when changing the schema; validate
  conversion and restoration on a copy before coordinating production changes.
  The maintained interface is `docs/annotation/label-studio.xml`; verify the
  deployed Label Studio version before changing controls or recommending upgrades.
- See [current research architecture](docs/architecture.md) for development
  targets, data meanings, current code, and scientific validation boundaries.

## Rust And Burn

- Prefer existing Burn and Rust ecosystem implementations for training, inference,
  and data processing. Add a small project-specific adapter where sufficient;
  keep training code, configuration, and evaluation reproducible in this repo.
- The project owner, GitHub `bioinformatist`, is an early Burn contributor with
  substantial prior contributions and established maintainer relationships.
  Upstream contributions are an intended outcome of this work, not a last resort.
- For a necessary general-purpose gap, identify the affected versions, rule out
  existing support, and provide a minimal reproducer or focused test before
  proposing a Burn/community PR. Prefer an upstreamable fix over a large private
  reimplementation. Treat actual PR publication as a separate authorized action;
  maintainer relationships do not guarantee acceptance or release timing.

## Measurement Evidence

- Fruit dimensions and whole-fruit fruitlet count are equally important. Evaluate
  their errors and availability independently; completion is not validity.
- AprilTag 36h11 uses the measured black-square side, excluding the outer white
  margin. Keep the student protocol simple: a flat tag beside the widest part of
  the fruit, near it, not in front or behind, and facing the camera.
- Treat a tag plane near the fruit axis as an approximation to validate against
  physical measurements, not evidence of recovered 3D shape. Accurate masks and
  tag decoding alone do not establish metric accuracy or unseen fruitlet count.
- Distinguish published benchmark conditions from this project's measurements.
  Parameter count, file size, and native-device speed do not establish browser
  memory, latency, quantization accuracy, or Burn compatibility.
