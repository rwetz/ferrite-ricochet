# Ricochet
Read ../ferrite-design/AGENTS.md before UI edits. Use palette roles, Ferrite chrome, ASCII framing and stable IDs. Never hardcode view colors. Physics lives in src/physics.rs and rendering in src/table.rs. Keep Q/E flippers and charge/release Space. Verify physics tests, formatting, clippy, release binaries and rendered UI before claiming completion.

## Ball clearance and future models
- Decorative toys never supply collision geometry. Keep raised scenery behind the playable top boundary or outside the side rails; playfield artwork is flush.
- Physical routes, gates and guides must be authored from the collision definitions. Check both open and closed gate states, including a ball entering from outside the intended shot.
- Leave at least 10 world units of clear passage after subtracting wall/gate thickness: the ball is 8 units across. Test both endpoints and adjacent angled guides. The reported Switchyard left gate pocket has a regression replay.
- Run all-map launch replays, fast/weak rail checks and closed-gate clearance tests after layout/model changes. Do not ship new decorative collision surfaces without new trajectory tests.
- Ball search is a fail-safe for stalled spatial progress, not a substitute for removing geometric traps. Captures and pauses must not trigger recovery. Do not award points or consume balls on recovery.

## Workspace conventions

Added by the 2026-10-30 workspace audit. Workspace-level rules live in `Dev/AGENTS.md`; this section only records how they apply here.

- **Registry.** This project has a row in `Dev/PROJECTS.md`. Update it in the same change as any move, rename or archive.
- **File header.** New source files a human authors start with the Ryan Wetzstein box (project line: `Ricochet`, year 2026) in the language's comment syntax, above imports and below any shebang. Skip generated files, lockfiles, JSON, Markdown and vendored code.
- **Builds.** Do not run `cargo build`, `cargo run` or `cargo clippy` locally: GPUI compiles overheat the user's CPU. Let CI build, then test the release `.exe`. This overrides any local build or check command above.
- **CI.** Linux-only, `pull_request` and `main` pushes, `concurrency` with `cancel-in-progress`, `paths-ignore` for docs (start from `_templates/workflows/ci-rust.yml`).
- **README.** Ferrite style, modeled on `ferrite/ferrite-wireless/README.md`: centered 128px `assets/logo.svg`, badge row, real `docs/main.png` screenshot, `<kbd>` keycaps, centered heptagram footer.
- **Jev.** Bounded judgments (pick one, score, yes/no) go through the TypeSafe skill or the contract in `apps/job-app-finder/src/job_app_finder/jev_fit.py`. Jev only ranks, routes or flags; irreversible actions stay behind human confirmation.
