# Ricochet
A native Ferrite pinball machine with a perspective 3D cabinet, raised mechanisms, machined steel and ASCII instruments. Built with Rust and GPUI. No webview.

## Play
Choose a machine on the animated welcome screen, select a three- or five-ball round, and press **Start round** or **Enter**. The preview switches with the board and Ferrite theme. The Scores and Rules tabs show local rankings and the scoring guide.

Hold **Q** and **E** for the flippers. Hold **Space** to charge and release to launch. **N** nudges, **P** pauses, **R** starts another round, **F** toggles effects and **M** mutes Windows audio. **Tab** gives the table the full play view. **Escape** returns to machine selection. **[ / ]** cycle maps; **Ctrl+Shift+P** opens commands. Click the table to return keyboard focus.

Every field is 35% wider and 18% longer than the original, approximately 59% more playable area. The ball, bumper cores, target faces and rail widths keep their size. Flippers, shooter routing, cabinet and camera follow the expanded layout. A six-second ball save protects each plunge. Closed gates leave an outside escape lane, and ball search releases persistent stalls without consuming a ball or awarding points.

### Scoring

| Shot | Base award |
| --- | ---: |
| Bumper | 100 x combo |
| Sling | 25 |
| Rollover | 75 |
| Drop target | 150 |
| Bumper bank | 750, then bank multiplier rises |
| Lane bank / drop-target bank | 500 |
| Orbit | 1,000 |
| Crossover | 1,500 + 250 per completed crossover |
| Reactor / overdrive activation | 2,000 |
| Jackpot | 5,000 + 1,000 per completed crossover |

Chain bumper hits within 2.5 seconds for up to 5x combo. The bank multiplier multiplies all awards, up to 5x. Overdrive doubles all awards for 20 seconds. The main sequence is bumper bank, inertia-driven orbit, then reactor. The separate drop-target sequence opens the crossover and lights growing jackpots. Weak rail shots roll back.

After the final ball, a results screen shows the score, weighted breakdown, round statistics and top-five rank. Only completed player rounds enter local rankings. Each board and ball limit has a separate leaderboard. V5 scoring uses a new leaderboard store; earlier records remain visible as legacy records. Attract mode, profiling and UI previews never save scores.

## Install
Download the Windows setup installer or portable ZIP from Releases. Lodestone discovers this repository through the `ferrite-app` topic and installs its platform archive after checking `SHA256SUMS.txt`.

## Develop
`cargo run` (Rust stable). `cargo run -- --smoke` verifies a deterministic launch through the scoring bank without opening a window. `cargo test` exercises physics, drains, launch routing and flipper velocity transfer. `cargo clippy --all-targets -- -D warnings` and `cargo fmt --all -- --check` are the CI gates. Launch in another scheme with `FERRITE_SCHEME=phosphor`; light mode with `FERRITE_APPEARANCE=light`.

Physics uses fixed substeps up to 240 Hz, a bounded catch-up interval, segment contacts, moving flipper surfaces, restitution and velocity limits. Rendering uses actual 3D vertices, perspective projection, quantised material faces and depth-sorted GPUI GPU paths. Static geometry is projected and tessellated only after a resize; live frames update moving mechanisms and effects. It does not rasterise or upload a complete CPU texture every frame. This is a purpose-built fixed-camera mesh renderer, without a general-purpose engine or depth buffer. Colors and chrome come directly from ferrite-design. Reduced motion disables impact shake, bursts and trails.

Release automation publishes a new patch release on every main push. Releases include Windows x64, Linux x64 and macOS Apple Silicon binaries.


## Measure

`cargo run --release -- --benchmark` compares the frozen v0.1 native raster workload against dynamic geometry generation/projection on the same deterministic replay. This is a historical raster-work comparison; the old raster uses the original dimensions. Use `--benchmark-all` for current wide-table preparation costs. Both explicitly exclude GPU presentation. `benches/driver.rs` can also be compiled with `rustc --edition 2024 -O` without building GPUI.

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

Use the header arrows, **[ / ]**, or **Ctrl+Shift+P** to select a map. Scheme selection now selects its matching layout and starts a new round. Each map and ball limit has its own rankings; earlier records remain visible separately. Light/dark changes keep the current round.

Each table also has a numbered drop-target bank. Hit the displayed order to open the crossover gate. A wrong target raises the bank again. A sufficiently strong ramp shot climbs the crest and lights a reactor jackpot; a weak shot rolls back. Completing another bank/ramp cycle increases the jackpot. This side sequence is independent of the bumper/orbit/overdrive sequence.

The maps also have their own 3D cabinet toys and artwork. Ferrous carries a steel flywheel gantry; Monolith has obelisks; Foundry has a hydraulic forging press; Breakwater has a lighthouse; Switchyard has a locomotive and signal tower; Drydock has a crane and cargo ship; Blueprint has a drafting arm and wireframe globe; Critical has a containment reactor and capacitor coils; Oxide has a valve pump and metal foliage; Blackout has speaker stacks and a lightning sculpture. Bumper hardware, etched emblems and apron stencils match each machine. Cabinet lamps report actual progression. Raised scenery sits behind the top boundary or outside the side rails, while playfield artwork stays flush.

`cargo run --release -- --benchmark-all` records static geometry build and dynamic preparation across every layout. `--smoke` runs a deterministic launch replay on all ten maps without a window. See [the catalog preview](docs/map-catalog.png) and [measurements](docs/PERFORMANCE.md).

## Asset format and model safety

The current assets are procedural polygon meshes in Rust, with actual 3D positions, camera projection and quantized Ferrite palette materials. They are not imported GLB/FBX files and do not use PBR textures or a general-purpose depth buffer. Blender would be a better authoring workflow for more intricate toys, but the game needs a model-import path before those files can replace these meshes. Decorative render meshes and simple authored collision geometry must remain separate. See `AGENTS.md` for gate-clearance and trajectory requirements.

`--preview-results` finishes a real simulated round and opens its result view for UI inspection without saving scores.
