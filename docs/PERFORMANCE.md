# Performance and design review

## Why v0.1 lagged
The original table recalculated 92,720 pixels for every frame, checked each against rails, circles and trail geometry, upscaled that buffer, allocated another render image and replaced its GPU texture. Physics was not the expensive part. The visual design also made depth impossible to read: rings and lines stood in for physical mechanisms, and the control panel competed with the table.

## Iterations
1. Replaced the raster with perspective 3D vertices and a camera. Added an extruded cabinet, rails, bumper stacks, posts, flippers and a faceted sphere. A geometry snapshot caught a reversed camera-up basis and an incorrect floor-layer order; both were corrected before the native build.
2. Cached projected/tessellated static paths. Batched contiguous floor markings so hundreds of etched dots share a GPU path. Only moving geometry and impact effects are rebuilt during play.
3. Reworked the score panel and added circuit goals, return lanes, combos, ball save and mechanical audio. The shooter-lane exit now redirects velocity without teleporting the ball across the table.

## Repeatable CPU measurement
The optimized standalone Rust driver replays 420 frames, drops 60 warmup frames, then runs both render workloads against the exact same game states with effects on. The baseline is the frozen native raster logic from commit `0502877`; it excludes its additional resize and GPU upload. The candidate measures dynamic mesh generation, camera projection and depth sorting; static table construction is amortized. Neither workload includes GPUI tessellation/submission or GPU completion.

On this Windows machine, the second candidate measured:

| CPU work per frame | Mean | p95 |
| --- | ---: | ---: |
| Frozen native raster | 5.537 ms | 6.021 ms |
| Dynamic 3D geometry and projection | 0.067 ms | 0.081 ms |
| Physics tick | 0.00043 ms | 0.00050 ms |

The render preparation comparison is about 83x faster in this run. This is not an 83x FPS claim. See `benchmark-first.json` and `benchmark-candidate.json` for raw results. Results depend on machine load.

## Native measurement
The `--profile` mode measures the entire table paint callback, including dynamic tessellation and GPUI path submission, and frame-to-frame paint intervals. It uses a silent scripted 20-second game and discards the first two seconds. These intervals describe app frame cadence, not GPU presentation timestamps. The profile file reports percentiles, outliers and maximum draw count.

## Boundaries
This renderer is a fixed-camera 3D mesh renderer inside GPUI, with painter-order visibility rather than a general engine/depth buffer. It preserves the shared Ferrite shell, scheme tokens, pixel type and stepped hit effects. Physical metal highlights come from discrete material roles; there are no texture gradients or full-frame ASCII filters. Mechanical audio currently uses the Windows system mixer; other platform builds retain visual feedback.
