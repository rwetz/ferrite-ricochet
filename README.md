# Ricochet
Native Ferrite pinball built with Rust and GPUI. Dithered steel, ASCII instruments, stepped screen shake, impact rings and ball trails. No webview.

## Play
Hold **Q** and **E** for the left and right flippers. Hold **Space** to charge, release to launch. **N** nudges the table; repeated nudges disable flippers through tilt. **P** pauses, **R** starts a new game, **F** toggles impact effects. **Ctrl+Shift+P** opens commands and all ten Ferrite schemes. Click the table to return keyboard focus.

Three balls. Bumpers award 100 points, slings 25. Light the entire bumper bank for 500 and raise the multiplier up to 5x. High water is saved locally between sessions.

## Install
Download the Windows setup installer or portable ZIP from Releases. Lodestone discovers this repository through the `ferrite-app` topic and installs its platform archive after checking `SHA256SUMS.txt`.

## Develop
`cargo run` (Rust stable). `cargo run -- --smoke` verifies a deterministic launch through the scoring bank without opening a window. `cargo test` exercises physics, drains, launch routing and flipper velocity transfer. `cargo clippy --all-targets -- -D warnings` and `cargo fmt --all -- --check` are the CI gates. Launch in another scheme with `FERRITE_SCHEME=phosphor`; light mode with `FERRITE_APPEARANCE=light`.

Physics uses fixed substeps up to 240 Hz, a bounded catch-up interval, segment contacts, moving flipper surfaces, restitution and velocity limits. Rendering is a custom GPUI pixel canvas with ordered Bayer dither. Colors and chrome come directly from ferrite-design. Reduced motion disables impact shake, bursts and trails.

Release automation publishes a new patch release on every main push. Releases include Windows x64, Linux x64 and macOS Apple Silicon binaries.

