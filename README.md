# Ricochet
A native Ferrite pinball machine with a perspective 3D cabinet, raised mechanisms, machined steel and ASCII instruments. Built with Rust and GPUI. No webview.

## Play
Hold **Q** and **E** for the left and right flippers. Hold **Space** to charge, release to launch. **N** nudges the table; repeated nudges disable flippers through tilt. **P** pauses, **R** starts a new game, **F** toggles impact effects. **M** mutes synthesized mechanical audio on Windows. **Ctrl+Shift+P** opens commands and all ten Ferrite schemes. Click the table to return keyboard focus.

Choose a three- or five-ball round in the command menu, with a six-second ball save after each plunge. Bumpers award 100 points, slings 25, return lanes 50. Chain bumper hits within 2.5 seconds for combos up to 5x. Complete the bumper bank for 500 and a higher bank multiplier, or the three lanes for 250. High water is saved locally between sessions.

The main sequence on every table: light the bumper bank to open the orbit gate, carry enough momentum through the elevated curved rail, then feed the central reactor scoop. Completing the sequence pays 750 for the orbit and 1,000 for the scoop, then doubles all scoring for 20 seconds. Weak rail shots roll back instead of completing automatically.

## Install
Download the Windows setup installer or portable ZIP from Releases. Lodestone discovers this repository through the `ferrite-app` topic and installs its platform archive after checking `SHA256SUMS.txt`.

## Develop
`cargo run` (Rust stable). `cargo run -- --smoke` verifies a deterministic launch through the scoring bank without opening a window. `cargo test` exercises physics, drains, launch routing and flipper velocity transfer. `cargo clippy --all-targets -- -D warnings` and `cargo fmt --all -- --check` are the CI gates. Launch in another scheme with `FERRITE_SCHEME=phosphor`; light mode with `FERRITE_APPEARANCE=light`.

Physics uses fixed substeps up to 240 Hz, a bounded catch-up interval, segment contacts, moving flipper surfaces, restitution and velocity limits. Rendering uses actual 3D vertices, perspective projection, quantised material faces and depth-sorted GPUI GPU paths. Static geometry is projected and tessellated only after a resize; live frames update moving mechanisms and effects. It does not rasterise or upload a complete CPU texture every frame. This is a purpose-built fixed-camera mesh renderer, without a general-purpose engine or depth buffer. Colors and chrome come directly from ferrite-design. Reduced motion disables impact shake, bursts and trails.

Release automation publishes a new patch release on every main push. Releases include Windows x64, Linux x64 and macOS Apple Silicon binaries.


## Measure

`cargo run --release -- --benchmark` compares the frozen v0.1 native raster workload against dynamic geometry generation/projection on the same deterministic replay. It reports p50/p95/p99 CPU timings and explicitly excludes GPU presentation. `benches/driver.rs` can also be compiled with `rustc --edition 2024 -O` without building GPUI.

`ferrite-ricochet --profile profile.json` runs a 20-second silent attract-mode session, writes live CPU paint timings and observed frame cadence after a two-second warmup, then exits. Frame cadence is not a GPU presentation timestamp. `--demo` runs attract mode without exiting; New game returns to manual play.

See [performance and design notes](docs/PERFORMANCE.md) for measurements and their limits.

## Ten authored maps

| Scheme | Map | Character |
| --- | --- | --- |
| Ferrite | Ferrous | Original factory circuit, left orbit and sweeping right crossover |
| Mono | Monolith | Mirrored orbit, symmetric bumper bank and left crossover |
| Graphite | Foundry | Staggered machinery and a diagonal center-spanning ramp |
| Slate | Breakwater | Winding wave-like orbit and a wide, rising crossover |
| Concrete | Switchyard | Split guides and a ramp threading between offset bumper cores |
| Harbor | Drydock | Sweeping hull routes and an asymmetric dockside bank |
| Cyanotype | Blueprint | Right orbit, left crossover and an etched engineering grid |
| Phosphor | Critical | Tight core arrangement and a reactor-style sweeping ramp |
| Verdigris | Oxide | Braided routes, offset cores and a descending target order |
| Bruise | Blackout | Tight bends, offset targets and an angular crossover |

Use the header arrows, **[ / ]**, or **Ctrl+Shift+P** to select a map. Scheme selection now selects its matching layout and starts a new round. Each map has its own record; your original high score is retained for Ferrite. Light/dark changes keep the current round.

Each table also has a numbered drop-target bank. Hit the displayed order to open the crossover gate. A wrong target raises the bank again. A sufficiently strong ramp shot climbs the crest and lights a reactor jackpot; a weak shot rolls back. Completing another bank/ramp cycle increases the jackpot. This side sequence is independent of the bumper/orbit/overdrive sequence.

`cargo run --release -- --benchmark-all` records static geometry build and dynamic preparation across every layout. `--smoke` runs a deterministic launch replay on all ten maps without a window. See [the catalog preview](docs/map-catalog.png) and [measurements](docs/PERFORMANCE.md).
