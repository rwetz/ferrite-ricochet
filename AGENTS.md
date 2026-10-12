# Ricochet
Read ../ferrite-design/AGENTS.md before UI edits. Use palette roles, Ferrite chrome, ASCII framing and stable IDs. Never hardcode view colors. Physics lives in src/physics.rs and rendering in src/table.rs. Keep Q/E flippers and charge/release Space. Verify physics tests, formatting, clippy, release binaries and rendered UI before claiming completion.

## Ball clearance and future models
- Decorative toys never supply collision geometry. Keep raised scenery behind the playable top boundary or outside the side rails; playfield artwork is flush.
- Physical routes, gates and guides must be authored from the collision definitions. Check both open and closed gate states, including a ball entering from outside the intended shot.
- Leave at least 10 world units of clear passage after subtracting wall/gate thickness: the ball is 8 units across. Test both endpoints and adjacent angled guides. The reported Switchyard left gate pocket has a regression replay.
- Run all-map launch replays, fast/weak rail checks and closed-gate clearance tests after layout/model changes. Do not ship new decorative collision surfaces without new trajectory tests.
- Ball search is a fail-safe for stalled spatial progress, not a substitute for removing geometric traps. Captures and pauses must not trigger recovery. Do not award points or consume balls on recovery.
