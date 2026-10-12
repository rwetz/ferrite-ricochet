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
The first native 3D candidate measured 32.47 observed FPS, p95 paint interval 33.76 ms, and p95 table CPU paint time 1.14 ms. That showed that fast render preparation alone did not fix cadence. The background 16 ms timer was replaced with GPUI window-frame callbacks. The final profile is recorded separately below.

The `--profile` mode measures the entire table paint callback, including dynamic tessellation and GPUI path submission, and frame-to-frame paint intervals. It uses a silent scripted 20-second game and discards the first two seconds. These intervals describe app frame cadence, not GPU presentation timestamps. The profile file reports percentiles, outliers and maximum draw count.

## Boundaries
This renderer is a fixed-camera 3D mesh renderer inside GPUI, with painter-order visibility rather than a general engine/depth buffer. It preserves the shared Ferrite shell, scheme tokens, pixel type and stepped hit effects. Physical metal highlights come from discrete material roles; there are no texture gradients or full-frame ASCII filters. Mechanical audio currently uses the Windows system mixer; other platform builds retain visual feedback.

## Progressive table pass
Ferrite now has an elevated sampled Bezier orbit, a physical gate, a staged reactor scoop and a 20-second overdrive. The rail simulation conserves forward/rollback momentum under table gravity and drag. A fast shot completes; a weak one stalls and rolls back. Three- and five-ball rounds are selectable. Other themes currently resolve to the authored Ferrite board, with a stable board-identity/cache seam ready for future layouts.

## Verified live result after frame-clock correction
On Windows with a Ryzen 7 9800X3D, the 20-second native session (two-second warmup excluded) recorded 1,081 measured frames, 60.003 observed FPS, p95 paint interval 17.274 ms and no interval above 20 ms. Table CPU paint time was 1.578 ms at p95 and 2.004 ms maximum, with up to 1,800 submitted paths. The first live mesh candidate had measured 32.470 observed FPS and 33.756 ms p95 intervals. The frame-clock correction therefore addressed a separate scheduling ceiling as well as the raster bottleneck.

The progressive table’s standalone replay measured 5.561 ms mean / 6.056 ms p95 for the old native raster versus 0.058 ms mean / 0.077 ms p95 for dynamic mesh preparation. Full-table static geometry is built once, and contains 2,506 faces. These CPU preparation timings exclude tessellation/GPU submission and are not a FPS multiplier.

The native run used the remembered default window size, effects on, the Ferrite dark scheme and silent attract mode. Frame cadence remains an app paint observation, rather than a hardware GPU presentation measurement. Other software on this machine and different window sizes can change the results. Raw files: `native-profile-before-frame-clock.json`, `native-profile-frame-clock.json`, `benchmark-progressive.json`.

## v0.3 map and ramp pass
All ten schemes now resolve to distinct authored board definitions, including different bumper layouts, guides, lane positions, ordered target banks, reactor positions, orbit curves and crossover curves. The second ramp has a solid elevated deck and supports. Wrong-order target contacts reset the bank; a completed sequence physically opens its ramp gate. Strong shots crest and light jackpots; weak shots roll back. Rail acceleration includes table gravity, height gradient and drag. Static geometry remains cached.

The all-map CPU fixture builds each layout once, then measures 360 dynamic geometry/projection samples after 60 warmup samples, including alternating flippers, bumper flashes and ramp traversal. It is a synthetic render fixture rather than GPU presentation telemetry. Raw results are in `benchmark-all-maps.json`: static build costs were below 0.6 ms and p95 dynamic preparation below 0.07 ms in this run. Native cadence is validated separately on the shipped binary.
