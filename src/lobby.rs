//! Welcome, machine selection and completed-round results share the Ferrite
//! shell. Previews use the same mesh renderer as play and animate on selection.
use crate::{Ricochet, Screen, physics::Game, table};
use ferrite_design::prelude::*;
use gpui::{AnyElement, Context, IntoElement, Window, div, px};

fn description(key: &str) -> &'static str {
    match key {
        "mono" => "Symmetric cores. A mirrored orbit beneath the monoliths.",
        "graphite" => "Staggered machinery. Diagonal shots through the forging press.",
        "slate" => "Ride the wave-shaped orbit beneath the north lighthouse.",
        "concrete" => "Split guides, offset cores and a winding night-freight crossover.",
        "harbor" => "Sweep the dockside circuit beneath the crane and cargo ship.",
        "cyanotype" => "Precision lines. A right orbit and left crossover under the drafting rig.",
        "phosphor" => "Tight reactor cores. Climb the containment crossover.",
        "verdigris" => "Braided routes and a descending target order beneath the valve pump.",
        "bruise" => "Angular turns. Offset targets and a high-voltage crossover.",
        _ => "Forged steel, a sweeping orbit and the original flywheel gantry.",
    }
}
pub(super) fn screen(
    this: &Ricochet,
    window: &mut Window,
    cx: &mut Context<Ricochet>,
) -> AnyElement {
    let p = palette(cx);
    let results = this.screen == Screen::Results;
    let g = &this.game;
    let preview = if results {
        g.clone()
    } else {
        Game {
            board: g.board,
            ball_limit: g.ball_limit,
            ..Game::default()
        }
    };
    let heading = if results {
        if this.rank == Some(1) {
            "NEW HIGH SCORE"
        } else {
            "ROUND COMPLETE"
        }
    } else {
        "RICOCHET"
    };
    let left = div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .gap_2()
        .child(div().display(Scale::X2, window).child(g.board.title))
        .child(
            div()
                .body(text::SM)
                .text_color(hsla(p.fg_dim))
                .child(description(g.board.key)),
        )
        .child(
            div()
                .relative()
                .flex_1()
                .min_h_0()
                .bg(hsla(p.sunken))
                .child(
                    div().absolute().inset_0().child(develop(
                        "machine-preview",
                        (g.board.key, this.result_id, results),
                        div()
                            .w(px(
                                (f32::from(window.viewport_size().width) - 388.).max(120.)
                            ))
                            .h(px(
                                (f32::from(window.viewport_size().height) - 210.).max(140.)
                            ))
                            .child(table::table(preview, false, cx)),
                    )),
                ),
        )
        .child(
            div()
                .body(text::SM)
                .text_color(hsla(p.fg_dim))
                .child("Q / E flippers   Hold Space to plunge   N nudge"),
        )
        .child(
            div()
                .body(text::SM)
                .text_color(hsla(p.fg_dim))
                .child("Tab expands the play view. Escape returns to the machines."),
        );
    let mut content = div()
        .id("lobby-content")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap_3();
    if results {
        content = content
            .child(ascii_rule(Some("FINAL SCORE")))
            .child(
                div()
                    .display(Scale::X3, window)
                    .text_color(hsla(p.accent_text))
                    .child(format!("{:08}", g.score)),
            )
            .child(div().body(text::BASE).child(if this.ui_preview {
                "Preview round / score not saved".into()
            } else if let Some(rank) = this.rank {
                format!("#{} in {}-ball local rankings", rank, g.ball_limit)
            } else {
                "Finish in the top five to enter the rankings.".into()
            }))
            .when(this.record_error, |el| {
                el.child(
                    div()
                        .body(text::SM)
                        .text_color(hsla(p.danger))
                        .child("Score could not be saved. Your final score is shown above."),
                )
            })
            .child(ascii_rule(Some("SCORE BREAKDOWN")));
        for (i, name) in ["CONTACTS", "CIRCUITS", "RAIL SHOTS", "REACTOR"]
            .into_iter()
            .enumerate()
        {
            content = content.child(
                div()
                    .flex()
                    .justify_between()
                    .body(text::BASE)
                    .child(name)
                    .child(format!("{:08}", g.score_parts[i])),
            );
        }
        content = content.child(
            div()
                .body(text::SM)
                .text_color(hsla(p.fg_dim))
                .child(format!(
                    "{} bumper hits / {} crossovers / {} jackpots / {:02}:{:02}",
                    g.hits,
                    g.ramp_runs,
                    g.jackpots,
                    g.time as u32 / 60,
                    g.time as u32 % 60
                )),
        );
        content = content.child(ascii_rule(Some("LOCAL TOP FIVE")));
    } else {
        content = content.child(
            tabs("lobby-tabs")
                .tab("Machines")
                .tab("Scores")
                .tab("Rules")
                .selected(this.lobby_tab)
                .on_select(cx.listener(|this, index, window, cx| {
                    this.lobby_tab = *index;
                    this.focus.focus(window, cx);
                    cx.notify();
                })),
        );
        if this.lobby_tab == 0 {
            for pair in SCHEMES.chunks(2) {
                let mut row = div().flex().gap_2();
                for scheme in pair {
                    let index = SCHEMES.iter().position(|s| s.key == scheme.key).unwrap();
                    let spec = crate::board::for_scheme(scheme.key);
                    row = row.child(
                        div().flex_1().min_w_0().child(
                            Button::new(spec.key)
                                .label(spec.title)
                                .secondary()
                                .selected(spec.key == g.board.key)
                                .full_width()
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.choose_board(index, cx);
                                    this.focus.focus(window, cx);
                                    cx.notify();
                                })),
                        ),
                    );
                }
                content = content.child(row);
            }
            content =
                content
                    .child(div().body(text::BASE).text_color(hsla(p.fg_dim)).child(
                        "Each machine has its own routes, target order, artwork and rankings.",
                    ))
                    .child(ascii_rule(Some("YOUR RECORD")))
                    .child(
                        div()
                            .display(Scale::X2, window)
                            .child(format!("{:08}", this.scores.first().map_or(0, |e| e.score))),
                    )
                    .child(
                        div()
                            .body(text::SM)
                            .text_color(hsla(p.fg_dim))
                            .child(format!("{}-ball completed rounds", g.ball_limit)),
                    )
                    .when(this.legacy > 0, |el| {
                        el.child(
                            div()
                                .body(text::SM)
                                .text_color(hsla(p.fg_faint))
                                .child(format!(
                                    "Legacy record: {} / previous scoring rules",
                                    this.legacy
                                )),
                        )
                    });
        } else if this.lobby_tab == 1 {
            content = content.child(ascii_rule(Some("LOCAL TOP FIVE")));
        } else {
            content = content.child(ascii_rule(Some("BUILD YOUR SCORE")));
            for (name, value) in [
                ("Bumper", "100 x combo"),
                ("Sling", "25"),
                ("Rollover", "75"),
                ("Drop target", "150"),
                ("Bumper bank", "750 + bank multiplier"),
                ("Lanes / target bank", "500"),
                ("Orbit", "1,000"),
                ("Crossover", "1,500 + 250 per run"),
                ("Reactor / overdrive", "2,000"),
                ("Jackpot", "5,000 + 1,000 per run"),
            ] {
                content = content.child(
                    div()
                        .flex()
                        .justify_between()
                        .gap_2()
                        .body(text::SM)
                        .child(name)
                        .child(value),
                );
            }
            content=content.child(div().body(text::BASE).text_color(hsla(p.fg_dim)).child("Chain bumper hits within 2.5 seconds for up to 5x combo. Each bumper bank raises the bank multiplier, up to 5x. Overdrive doubles every award for 20 seconds."))
                .child(div().body(text::SM).text_color(hsla(p.fg_dim)).child("Finish the bumper bank, orbit and reactor sequence for overdrive. Hit drop targets in order, then complete the crossover to light jackpots. Three- and five-ball scores rank separately."));
        }
    }
    if results || this.lobby_tab == 1 {
        if this.scores.is_empty() {
            content = content.child(
                div()
                    .body(text::BASE)
                    .text_color(hsla(p.fg_dim))
                    .child("No ranked rounds yet. Set the first record."),
            );
        }
        for (i, entry) in this.scores.iter().enumerate() {
            content = content.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .display(Scale::X1, window)
                            .child(format!("{:02}", i + 1))
                            .child(format!("{:08}", entry.score)),
                    )
                    .child(
                        div()
                            .body(text::SM)
                            .text_color(hsla(p.fg_dim))
                            .child(format!(
                                "{} hits / {} circuits / {} ramps / {} jackpots",
                                entry.hits, entry.circuits, entry.ramps, entry.jackpots
                            )),
                    ),
            );
        }
        content = content.child(
            div()
                .body(text::SM)
                .text_color(hsla(p.fg_dim))
                .child(format!(
                    "{} / {}-ball / local only",
                    g.board.title, g.ball_limit
                )),
        );
    }
    let right = div()
        .w(px(340.))
        .h(px(
            (f32::from(window.viewport_size().height) - 88.).max(220.)
        ))
        .flex_shrink_0()
        .flex()
        .flex_col()
        .min_h_0()
        .gap_3()
        .child(
            div()
                .display(Scale::X2, window)
                .text_color(hsla(if this.rank == Some(1) {
                    p.accent_text
                } else {
                    p.fg
                }))
                .child(heading),
        )
        .child(
            div()
                .body(text::SM)
                .text_color(hsla(p.fg_dim))
                .child(if results {
                    "The machine rests. The next round is yours."
                } else {
                    "TEN MACHINES. ONE MORE ROUND."
                }),
        )
        .child(content)
        .when(!results, |el| {
            el.child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        Button::new("three-balls")
                            .label("3 balls")
                            .secondary()
                            .selected(g.ball_limit == 3)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.choose_limit(3);
                                this.focus.focus(window, cx);
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("five-balls")
                            .label("5 balls")
                            .secondary()
                            .selected(g.ball_limit == 5)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.choose_limit(5);
                                this.focus.focus(window, cx);
                                cx.notify();
                            })),
                    ),
            )
        })
        .child(
            Button::new("start-round")
                .label(if results { "Play again" } else { "Start round" })
                .primary()
                .full_width()
                .on_click(cx.listener(|this, _, window, cx| {
                    this.new_game();
                    this.focus.focus(window, cx);
                    cx.notify();
                })),
        )
        .when(results, |el| {
            el.child(
                Button::new("result-choose-board")
                    .label("Choose board")
                    .secondary()
                    .full_width()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.show_welcome();
                        this.focus.focus(window, cx);
                        cx.notify();
                    })),
            )
        });
    window_frame()
        .child(power_on_in(
            "power",
            div()
                .id("lobby-root")
                .track_focus(&this.focus)
                .flex()
                .flex_col()
                .size_full()
                .bg(hsla(p.bg))
                .text_color(hsla(p.fg))
                .body(text::BASE)
                .on_action(cx.listener(|this, _: &TogglePalette, window, cx| {
                    this.palette.update(cx, |p, cx| p.toggle(window, cx))
                }))
                .on_key_down(cx.listener(|this, ev: &gpui::KeyDownEvent, window, cx| {
                    if !this.focus.is_focused(window) {
                        return;
                    }
                    if !ev.is_held && matches!(ev.keystroke.key.as_str(), "[" | "]") {
                        this.cycle_table(if ev.keystroke.key == "[" { -1 } else { 1 }, window, cx);
                    } else {
                        this.key(&ev.keystroke.key, true, ev.is_held);
                        cx.notify();
                    }
                }))
                .on_key_up(cx.listener(|this, ev: &gpui::KeyUpEvent, window, cx| {
                    if this.focus.is_focused(window) {
                        this.key(&ev.keystroke.key, false, false);
                        cx.notify();
                    }
                }))
                .child(title_bar("Ricochet"))
                .child(
                    div()
                        .flex()
                        .flex_1()
                        .min_h_0()
                        .gap_4()
                        .p_4()
                        .child(left)
                        .child(interlace_in("lobby-pane", (results, this.result_id), right)),
                )
                .child(
                    status_bar()
                        .left_live(if results {
                            "ENTER / PLAY AGAIN   ESC / MACHINES"
                        } else {
                            "SELECT A MACHINE   ENTER / START"
                        })
                        .right_live(format!(
                            "{}-BALL / [ ] SELECT / CTRL SHIFT P MENU",
                            g.ball_limit
                        )),
                )
                .child(this.palette.clone()),
        ))
        .into_any_element()
}
