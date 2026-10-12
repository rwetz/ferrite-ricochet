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

## v0.2 progressive table pass
Ferrite gained an elevated sampled Bezier orbit, a physical gate, a staged reactor scoop and a 20-second overdrive. Rail speed evolves under table gravity and drag. A fast shot completes; a weak one stalls and rolls back. Three- and five-ball rounds are selectable. This iteration established board identity and caching; v0.3 supplies a distinct layout for every scheme.

## Verified live result after frame-clock correction
On Windows with a Ryzen 7 9800X3D, the 20-second native session (two-second warmup excluded) recorded 1,081 measured frames, 60.003 observed FPS, p95 paint interval 17.274 ms and no interval above 20 ms. Table CPU paint time was 1.578 ms at p95 and 2.004 ms maximum, with up to 1,800 submitted paths. The first live mesh candidate had measured 32.470 observed FPS and 33.756 ms p95 intervals. The frame-clock correction therefore addressed a separate scheduling ceiling as well as the raster bottleneck.

The progressive table’s standalone replay measured 5.561 ms mean / 6.056 ms p95 for the old native raster versus 0.058 ms mean / 0.077 ms p95 for dynamic mesh preparation. Full-table static geometry is built once, and contains 2,506 faces. These CPU preparation timings exclude tessellation/GPU submission and are not a FPS multiplier.

The native run used the remembered default window size, effects on, the Ferrite dark scheme and silent attract mode. Frame cadence remains an app paint observation, rather than a hardware GPU presentation measurement. Other software on this machine and different window sizes can change the results. Raw files: `native-profile-before-frame-clock.json`, `native-profile-frame-clock.json`, `benchmark-progressive.json`.

## v0.3 map and ramp pass
All ten schemes now resolve to distinct authored board definitions, including different bumper layouts, guides, lane positions, ordered target banks, reactor positions, orbit curves and crossover curves. The second ramp has a solid elevated deck and supports. Wrong-order target contacts reset the bank; a completed sequence physically opens its ramp gate. Strong shots crest and light jackpots; weak shots roll back. Rail acceleration includes table gravity, height gradient and drag. Static geometry remains cached.

The all-map CPU fixture builds each layout once, then measures 360 dynamic geometry/projection samples after 60 warmup samples, including alternating flippers, bumper flashes and ramp traversal. It is a synthetic render fixture rather than GPU presentation telemetry. Raw results are in `benchmark-all-maps.json`: static build costs were below 0.6 ms and p95 dynamic preparation below 0.07 ms in this run. Native cadence is validated separately on the shipped binary.

## Shipped v0.3.0: native measurements across all maps

Each map ran in the checksum-verified Windows release binary for 20 seconds, with two seconds of warmup excluded. The table viewport was 672 x 732 logical pixels at 150% display scale, with impact effects enabled and silent attract mode. This machine uses a Ryzen 7 9800X3D. Every run produced 1,081 measured frames and zero paint intervals above 20 ms.

| Scheme | Observed FPS | CPU paint p95 (ms) | Paint interval p95 (ms) |
| --- | ---: | ---: | ---: |
| Ferrite | 60.001 | 2.207 | 17.210 |
| Mono | 60.003 | 2.319 | 17.229 |
| Graphite | 59.999 | 2.269 | 17.227 |
| Slate | 60.001 | 2.411 | 17.251 |
| Concrete | 60.000 | 2.309 | 17.228 |
| Harbor | 59.999 | 2.306 | 17.244 |
| Cyanotype | 60.002 | 2.367 | 17.209 |
| Phosphor | 59.999 | 2.503 | 17.212 |
| Verdigris | 59.999 | 2.377 | 17.205 |
| Bruise | 59.999 | 2.259 | 17.134 |

The busiest layout's p95 table CPU paint time was 2.503 ms. These measurements include the table paint callback and path submission, but exclude asynchronous GPU completion and GPU presentation timestamps. They demonstrate consistent app paint cadence on this machine and viewport, not a universal hardware frame-rate guarantee. Raw per-map JSON is bundled in [profiles-v0.3.0](profiles-v0.3.0/).

All ten shipped maps also passed deterministic launch replays. Automated checks cover rail continuity, fast-shot completion versus weak-shot rollback on every ramp and orbit, ordered contacts, jackpot progression, round limits, and scheme-to-map resolution. Native UI inspection verified the header map control, matching palette/layout changes and the five-ball round command.

## v0.4 decorative asset pass

Ten authored centerpieces, bumper hardware, cabinet service plates, vents, rivets, playfield emblems and stencils are built into the existing mesh renderer. There are no new image uploads, asset downloads or continuously animated decorative simulations. Raised toys sit outside the playable field; the new playfield artwork is flush. The camera reserves space for the taller silhouettes, and the same projection/bounds checks cover every new face.

The standalone all-map render fixture measured fewer than 4,400 static faces per layout, below 0.7 ms for one-time geometry construction/projection and below 0.07 ms p95 for dynamic preparation on this machine. Static geometry is cached at the native window's size. The only new per-frame geometry is six small progression lamps. Raw CPU fixture measurements are in `benchmark-decorations.json`; these preparation timings exclude native tessellation, submission and GPU presentation.

### Shipped v0.4.0: decorative meshes in native play

The checksum-verified Windows release binary ran the same silent 20-second attract scenario for every map, excluding two seconds of warmup. The viewport remained 672 x 732 logical pixels at 150% display scale on the Ryzen 7 9800X3D machine. Each map produced 1,081 measured frames and zero paint intervals above 20 ms.

| Scheme | Observed FPS | CPU paint p95 (ms) | Paint interval p95 (ms) |
| --- | ---: | ---: | ---: |
| Ferrite | 60.000 | 2.808 | 17.268 |
| Mono | 60.001 | 2.830 | 17.271 |
| Graphite | 60.002 | 2.869 | 17.253 |
| Slate | 60.001 | 2.856 | 17.265 |
| Concrete | 60.003 | 2.840 | 17.220 |
| Harbor | 60.000 | 2.719 | 17.237 |
| Cyanotype | 60.000 | 2.917 | 17.204 |
| Phosphor | 59.998 | 3.069 | 17.284 |
| Verdigris | 60.001 | 2.828 | 17.349 |
| Bruise | 60.000 | 2.898 | 17.370 |

The added static detail increases CPU paint work: Ferrite rose from 2.207 to 2.808 ms at p95 compared with v0.3.0. Observed frame cadence remained approximately 60 FPS on every map. These are app paint observations and CPU callback timings, not asynchronous GPU completion or presentation measurements. Raw JSON is in [profiles-v0.4.0](profiles-v0.4.0/).
