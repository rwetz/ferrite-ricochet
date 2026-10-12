# Changelog

## 0.2.0
- Rebuilt the table as perspective 3D geometry: cabinet, raised rails, bumper stacks, posts, faceted ball and solid flippers.
- Removed per-frame full-table CPU rasterisation, scaling and GPU texture churn. Static paths are cached and moving mechanisms update independently.
- Reworked the instrument panel around score, circuits, combo, lane goals, plunger and tilt.
- Added rollover scoring, timed combos, a six-second ball save, smooth shooter-lane exit and Windows synthesized audio.
- Added repeatable CPU benchmark, live paint/cadence profiler and attract mode.

## 0.1.0
- Initial native pinball table with Q/E flippers, chargeable plunger, bumper banks, multiplier, slings, nudge and tilt.
- Ferrite ASCII instruments, dithered table, impact shake, trails and bursts; all ten schemes and light/dark mode.
- Windows portable release, setup installer, checksums and Lodestone discovery.
