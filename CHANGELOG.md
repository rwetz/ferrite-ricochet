# Changelog

## [Unreleased]

- The machine readouts scroll in short windows instead of being cut off.

## 0.5.2
- Kept the ball/bank HUD row on one line in the narrower instrument panel and synchronized its scoring captions with V5 awards.

## 0.5.1
- Gave animated machine previews explicit viewport dimensions after native inspection exposed a collapsed animation wrapper.

## 0.5.0
- Expanded every physical table by 35% in width and 18% in length while retaining ball, bumper and target sizes. Tuned flippers, launcher routing and camera framing for the larger field.
- Added an animated welcome screen with board previews, machine selection, round modes, local rankings and scoring rules.
- Replaced the game-over strip with completed-round results, score breakdown, leaderboard rank, replay and board selection.
- Introduced V5 scoring with separate top-five leaderboards per board and three/five-ball mode. Only completed player rounds rank; legacy records remain visible separately.
- Removed the reported Switchyard closed-gate pocket, added clearance checks and spatial ball-search recovery. Added collision rules for future models.
- Added Tab table-focus mode and Escape navigation to the welcome screen.

## 0.4.1
- Bundled native measurements for all ten decorated maps: approximately 60 observed FPS, no measured paint intervals above 20 ms and at most 3.069 ms p95 CPU paint time on the test machine.

## 0.4.0
- Added ten bespoke 3D cabinet centerpieces: a flywheel gantry, monoliths, hydraulic press, lighthouse, locomotive, dock crane and cargo ship, wireframe drafting rig, reactor coils, oxidized pump and lightning sculpture with speaker stacks.
- Added theme-specific bumper rim hardware, playfield emblems, stenciled aprons, service hatches, vents, rivets and shot inlays.
- Added cabinet progression lamps tied to the bumper bank, target bank, crossover gate and jackpot.
- Kept raised decorations outside the playable field and cached their meshes; widened camera framing to contain the new silhouettes.

## 0.3.1
- Bundled the map catalog, performance report and raw measurements with every platform package and Windows installer.
- Documented native v0.3.0 measurements for all ten maps: approximately 60 observed FPS, with no paint intervals above 20 ms during the measured sessions.

## 0.3.0
- Ten distinct maps, one for every Ferrite scheme, with authored bumper layouts, orbit and ramp curves, guides, target orders, reactor positions and etched graphics.
- Added a second raised crossover ramp, ordered physical drop targets, reset-on-wrong-shot progression and reactor jackpots.
- Rail inertia now includes the gravitational cost of gaining height. Fast shots crest; weak shots stall and roll back.
- Added header map controls, bracket-key cycling, per-map records and safe separation of attract/profile scores from player records.
- Added all-map simulation, geometry validation, benchmarks and profile context (board, viewport and scale factor).

## 0.2.0
- Added progressive bumper-bank/orbit/reactor sequence, inertia-driven elevated rail, weak-shot rollback, overdrive and three/five-ball round selection.
- Separated board identity from scheme selection for future themed tables.
- Replaced background timer pacing with native window frame callbacks after live profiling exposed a roughly 32 FPS ceiling.
- Rebuilt the table as perspective 3D geometry: cabinet, raised rails, bumper stacks, posts, faceted ball and solid flippers.
- Removed per-frame full-table CPU rasterisation, scaling and GPU texture churn. Static paths are cached and moving mechanisms update independently.
- Reworked the instrument panel around score, circuits, combo, lane goals, plunger and tilt.
- Added rollover scoring, timed combos, a six-second ball save, smooth shooter-lane exit and Windows synthesized audio.
- Added repeatable CPU benchmark, live paint/cadence profiler and attract mode.

## 0.1.0
- Initial native pinball table with Q/E flippers, chargeable plunger, bumper banks, multiplier, slings, nudge and tilt.
- Ferrite ASCII instruments, dithered table, impact shake, trails and bursts; all ten schemes and light/dark mode.
- Windows portable release, setup installer, checksums and Lodestone discovery.
