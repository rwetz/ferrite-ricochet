#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod audio;
mod benchmark;
mod board;
mod physics;
mod profile;
mod records;
mod scene;
mod table;
use ferrite_design::prelude::*;
use gpui::{
    App, AppContext as _, Context, Entity, FocusHandle, Focusable, IntoElement, KeyBinding, Render,
    Subscription, Window, div, px, size,
};
use std::time::{Duration, Instant};
struct Ricochet {
    game: physics::Game,
    audio: audio::Audio,
    palette: Entity<CommandPalette>,
    focus: FocusHandle,
    charging: bool,
    effects: bool,
    demo: bool,
    record_valid: bool,
    last_frame: Instant,
    frame_pending: bool,
    _appearance: Subscription,
}
impl Drop for Ricochet {
    fn drop(&mut self) {
        if self.record_valid && !profile::enabled() {
            records::save(self.game.board, self.game.best);
        }
    }
}
impl Focusable for Ricochet {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
impl Ricochet {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let initial_board = board::for_scheme(theme::scheme(cx).key);
        let demo = std::env::args().any(|a| a == "--demo") || profile::enabled();
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        let palette = cx.new(|cx| CommandPalette::new(window, cx));
        let weak = cx.weak_entity();
        let mut commands = Vec::new();
        for (label, action) in [
            ("New game", 0),
            ("Pause / resume", 1),
            ("Launch ball", 2),
            ("Toggle impact effects", 3),
            ("Nudge table", 4),
            ("Toggle sound", 5),
            ("Toggle attract mode", 6),
            ("New 3-ball round", 7),
            ("New 5-ball round", 8),
        ] {
            let weak = weak.clone();
            commands.push(command(label).group("Machine").on_run(move |window, cx| {
                let _ = weak.update(cx, |this, cx| {
                    match action {
                        0 => this.new_game(),
                        1 => {
                            this.demo = false;
                            this.game.paused = !this.game.paused;
                        }
                        2 => {
                            this.game.paused = false;
                            this.game.launch();
                        }
                        3 => this.effects = !this.effects,
                        4 => this.game.nudge(),
                        5 => this.audio.enabled = !this.audio.enabled,
                        6 => {
                            this.demo = !this.demo;
                            if this.demo {
                                this.record_valid = false;
                            }
                        }
                        7 => {
                            this.game.set_ball_limit(3);
                            this.new_game();
                        }
                        _ => {
                            this.game.set_ball_limit(5);
                            this.new_game();
                        }
                    }
                    this.focus.focus(window, cx);
                    cx.notify();
                });
            }));
        }
        for scheme in SCHEMES {
            commands.push(
                command(format!(
                    "Map: {} / {}",
                    board::for_scheme(scheme.key).title,
                    scheme.name
                ))
                .group("Tables")
                .on_run(move |_, cx| theme::set_scheme(scheme, cx)),
            );
        }
        commands.push(
            command("Toggle light / dark")
                .group("Appearance")
                .on_run(|w, cx| {
                    let a = if theme::appearance(cx) == Appearance::Dark {
                        Appearance::Light
                    } else {
                        Appearance::Dark
                    };
                    theme::set_appearance(a, w, cx);
                }),
        );
        palette.update(cx, |p, cx| p.set_commands(commands, cx));
        focus.focus(window, cx);
        Self {
            game: physics::Game {
                board: initial_board,
                best: records::load(initial_board),
                ..physics::Game::default()
            },
            audio: audio::Audio::new(cfg!(windows) && !profile::enabled()),
            palette,
            focus,
            charging: false,
            effects: true,
            demo,
            record_valid: !demo,
            last_frame: Instant::now(),
            frame_pending: false,
            _appearance: theme::follow_system(window),
        }
    }
    fn active(&self) -> bool {
        (!self.game.paused || self.demo)
            && (self.demo
                || self.game.live
                || self.charging
                || self.game.left
                || self.game.right
                || self.game.shaking > 0.
                || (self.game.angles[0] - 0.42).abs() > 0.001
                || (self.game.angles[1] - (std::f32::consts::PI - 0.42)).abs() > 0.001)
    }
    fn advance(&mut self, dt: f32) {
        if self.game.paused && !self.demo {
            return;
        }
        let active = self.demo
            || self.game.live
            || self.charging
            || self.game.left
            || self.game.right
            || self.game.shaking > 0.
            || (self.game.angles[0] - 0.42).abs() > 0.001
            || (self.game.angles[1] - (std::f32::consts::PI - 0.42)).abs() > 0.001;
        if !active {
            return;
        }
        if self.demo {
            self.game.paused = false;
            if !self.game.live {
                if self.game.balls == 0 {
                    self.game.reset();
                }
                self.game.charge = 0.7;
                self.game.launch();
            }
            self.game.left = (self.game.time * 2.3).sin() > 0.2;
            self.game.right = (self.game.time * 1.9).cos() > 0.1;
        }
        let previous = (
            self.game.hits,
            self.game.score,
            self.game.balls,
            self.game.multiplier,
            self.game.mission,
            self.game.ramp_runs,
            self.game.jackpots,
        );
        let was_live = self.game.live;
        self.game.tick(dt, self.charging);
        if self.game.balls < previous.2 {
            self.audio.play(audio::Cue::Drain);
        } else if self.game.multiplier > previous.3
            || self.game.mission > previous.4
            || self.game.ramp_runs > previous.5
            || self.game.jackpots > previous.6
        {
            self.audio.play(audio::Cue::Bank);
        } else if self.game.hits > previous.0 {
            self.audio.play(audio::Cue::Bumper);
        } else if self.game.score > previous.1 {
            self.audio.play(audio::Cue::Sling);
        }
        if was_live && !self.game.live && self.record_valid && !profile::enabled() {
            records::save(self.game.board, self.game.best);
        }
    }
    fn sync_table(&mut self, spec: &'static board::Board) {
        if self.game.board.key != spec.key {
            if self.record_valid && !profile::enabled() {
                records::save(self.game.board, self.game.best);
            }
            self.game.set_board(spec);
            self.game.best = records::load(spec);
            self.charging = false;
        }
    }
    fn cycle_table(&mut self, direction: isize, window: &mut Window, cx: &mut Context<Self>) {
        let index = SCHEMES
            .iter()
            .position(|s| s.key == self.game.board.key)
            .unwrap_or(0);
        let next = (index as isize + direction).rem_euclid(SCHEMES.len() as isize) as usize;
        theme::set_scheme(&SCHEMES[next], cx);
        self.sync_table(board::for_scheme(SCHEMES[next].key));
        self.focus.focus(window, cx);
        cx.notify();
    }
    fn new_game(&mut self) {
        if self.record_valid && !profile::enabled() {
            records::save(self.game.board, self.game.best);
        }
        self.game.best = records::load(self.game.board);
        self.game.reset();
        self.demo = false;
        self.record_valid = !profile::enabled();
        self.charging = false;
    }
    fn key(&mut self, key: &str, down: bool, held: bool) {
        if self.demo && down && !held && matches!(key, "q" | "e" | "space" | "r" | "n") {
            self.new_game();
        }
        match key {
            "q" => {
                if down && !held && !self.game.paused {
                    self.audio.play(audio::Cue::Flipper);
                }
                self.game.left = down;
            }
            "e" => {
                if down && !held && !self.game.paused {
                    self.audio.play(audio::Cue::Flipper);
                }
                self.game.right = down;
            }
            "space" => {
                if down {
                    self.charging = true;
                } else {
                    let charged = self.charging;
                    self.charging = false;
                    let was_live = self.game.live;
                    if charged {
                        self.game.launch();
                    }
                    if !was_live && self.game.live {
                        self.audio.play(audio::Cue::Launch);
                    }
                }
            }
            "p" if down && !held => {
                self.demo = false;
                self.game.paused = !self.game.paused;
                self.game.left = false;
                self.game.right = false;
                self.charging = false;
            }
            "r" if down && !held => self.new_game(),
            "n" if down && !held => self.game.nudge(),
            "m" if down && !held => self.audio.enabled = !self.audio.enabled,
            "f" if down && !held => self.effects = !self.effects,
            _ => {}
        }
    }
}
impl Render for Ricochet {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !window.is_window_active() && !self.demo {
            self.game.left = false;
            self.game.right = false;
            self.charging = false;
            if self.game.live {
                self.game.paused = true;
            }
        }
        self.sync_table(board::for_scheme(theme::scheme(cx).key));
        if self.active() && !self.frame_pending {
            if self.last_frame.elapsed() > Duration::from_millis(80) {
                self.last_frame = Instant::now();
            }
            self.frame_pending = true;
            let weak = cx.weak_entity();
            window.on_next_frame(move |_, cx| {
                let _ = weak.update(cx, |this, cx| {
                    this.frame_pending = false;
                    let now = Instant::now();
                    let dt = now.duration_since(this.last_frame).as_secs_f32();
                    this.last_frame = now;
                    this.advance(dt);
                    cx.notify();
                });
            });
        }
        let p = palette(cx);
        let g = &self.game;
        let effects = self.effects && !motion::reduced(cx);
        let lights = |values: [bool; 3]| {
            div()
                .flex()
                .gap_3()
                .children(values.into_iter().enumerate().map(|(i, on)| {
                    div()
                        .display(Scale::X1, window)
                        .text_color(hsla(if on { p.accent_text } else { p.fg_dim }))
                        .child(format!("[{}{}]", i + 1, if on { "*" } else { "-" }))
                }))
        };
        let instrument = div()
            .id("instrument-scroll")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap_2()
            .child(ascii_rule(Some("SCORE")))
            .child(
                div()
                    .display(Scale::X3, window)
                    .text_color(hsla(p.accent_text))
                    .child(format!("{:08}", g.score)),
            )
            .child(
                div()
                    .body(text::SM)
                    .text_color(hsla(p.fg_dim))
                    .child(format!("BEST  {:08}", g.best)),
            )
            .child(div().display(Scale::X1, window).child(format!(
                "BALL {} / {}       {}X BANK",
                (g.ball_limit + 1 - g.balls).min(g.ball_limit),
                g.ball_limit,
                g.multiplier
            )))
            .child(
                div()
                    .display(Scale::X1, window)
                    .text_color(hsla(if g.combo > 1 { p.accent_text } else { p.fg_dim }))
                    .child(format!("{}X COMBO     {} HITS", g.combo.max(1), g.hits)),
            )
            .child(ascii_rule(Some("SEQUENCE")))
            .child(
                div()
                    .display(Scale::X1, window)
                    .text_color(hsla(p.accent_text))
                    .child(board::mission(g.board, g.mission).0),
            )
            .child(
                div()
                    .body(text::SM)
                    .text_color(hsla(p.fg_dim))
                    .child(board::mission(g.board, g.mission).1),
            )
            .when(g.overdrive > 0., |el| {
                el.child(
                    div()
                        .display(Scale::X1, window)
                        .child(format!("OVERDRIVE / {:02}S", g.overdrive.ceil() as u32)),
                )
            })
            .child(ascii_rule(Some("CROSSOVER")))
            .child(
                div()
                    .display(Scale::X1, window)
                    .text_color(hsla(p.accent_text))
                    .child(if g.jackpot_ready {
                        "JACKPOT LIT / REACTOR".into()
                    } else if g.ramp_open {
                        format!(
                            "RAMP OPEN / {} SHOT",
                            if g.board.ramp.points[0].x < 122. {
                                "LEFT"
                            } else {
                                "RIGHT"
                            }
                        )
                    } else {
                        format!(
                            "DROP {} > {} > {}",
                            g.board.order[0] + 1,
                            g.board.order[1] + 1,
                            g.board.order[2] + 1
                        )
                    }),
            )
            .child(lights(g.drops))
            .child(
                div()
                    .body(text::SM)
                    .text_color(hsla(p.fg_dim))
                    .child(if g.jackpot_ready {
                        "Collect at the reactor. Ramp again to grow the jackpot."
                    } else if g.ramp_open {
                        "Build speed, clear the crest, and light the jackpot."
                    } else {
                        "Hit the numbered bank in order. A wrong shot resets it."
                    }),
            )
            .child(ascii_rule(Some("CIRCUIT BREAKER")))
            .child(lights(g.lamps))
            .child(
                div()
                    .body(text::SM)
                    .text_color(hsla(p.fg_dim))
                    .child("100 EACH / +500 BANK"),
            )
            .child(ascii_rule(Some("RETURN LANES")))
            .child(lights(g.rollovers))
            .child(
                div()
                    .body(text::SM)
                    .text_color(hsla(p.fg_dim))
                    .child("50 EACH / +250 BANK"),
            )
            .child(ascii_rule(Some("MACHINE")))
            .child(ascii_gauge(g.charge).label("PULL").cells(10))
            .child(ascii_gauge(g.tilt.clamp(0., 1.)).label("TILT").cells(10));
        let side = div()
            .id("machine-hud")
            .w(px(280.))
            .min_h_0()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap_2()
            .p_2()
            .child(div().display(Scale::X2, window).child("RICOCHET"))
            .child(
                div()
                    .body(text::SM)
                    .text_color(hsla(p.fg_dim))
                    .child(format!("{} / SERIES 03", g.board.title)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("previous-table")
                            .icon(Icon::ChevronLeft)
                            .small()
                            .secondary()
                            .tooltip("Previous map / new round")
                            .on_click(
                                cx.listener(|this, _, window, cx| this.cycle_table(-1, window, cx)),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .display(Scale::X1, window)
                            .text_color(hsla(p.fg_dim))
                            .child(format!("{:02} / 10", g.board.design + 1)),
                    )
                    .child(
                        Button::new("next-table")
                            .icon(Icon::ChevronRight)
                            .small()
                            .secondary()
                            .tooltip("Next map / new round")
                            .on_click(
                                cx.listener(|this, _, window, cx| this.cycle_table(1, window, cx)),
                            ),
                    ),
            )
            .child(instrument)
            .child(
                div()
                    .id("live-machine-state")
                    .display(Scale::X1, window)
                    .text_color(hsla(if g.tilt >= 1. {
                        p.danger
                    } else if g.tilt > 0.6 {
                        p.warning
                    } else {
                        p.accent_text
                    }))
                    .child(if g.paused {
                        "PAUSED / P TO RESUME".into()
                    } else if g.tilt >= 1. {
                        "TILT / Q + E DISABLED".into()
                    } else if g.tilt > 0.6 {
                        "TILT WARNING / EASE OFF".into()
                    } else if g.rail_active {
                        match g.rail_kind {
                            physics::RailKind::Orbit => "ORBIT / CARRY MOMENTUM".into(),
                            physics::RailKind::Ramp => "RAMP / CLIMB THE CREST".into(),
                        }
                    } else if g.save_left > 0. {
                        format!("BALL SAVE {:02}", g.save_left.ceil() as u32)
                    } else if self.demo {
                        "ATTRACT MODE".into()
                    } else {
                        "Q + E / KEEP IT ALIVE".into()
                    }),
            )
            .child(div().body(text::SM).text_color(hsla(p.fg_dim)).children([
                "Q / E FLIPPERS   SPACE PLUNGER",
                "N NUDGE  P PAUSE  R NEW  F FX",
                "[ / ] MAP   CTRL SHIFT P MENU",
            ]))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        Button::new("new-game")
                            .label("New game")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.new_game();
                                this.focus.focus(window, cx);
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("pause")
                            .label(if g.paused { "Resume" } else { "Pause" })
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.demo = false;
                                this.game.paused = !this.game.paused;
                                this.charging = false;
                                this.game.left = false;
                                this.game.right = false;
                                this.focus.focus(window, cx);
                                cx.notify();
                            })),
                    ),
            );
        let board = div()
            .id("table")
            .relative()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .bg(hsla(p.sunken))
            .on_click(cx.listener(|this, _, window, cx| this.focus.focus(window, cx)))
            .child(table::table(g.clone(), effects, cx))
            .when(g.paused || g.balls == 0, |el| {
                el.child(
                    div()
                        .absolute()
                        .top(px(160.))
                        .left(px(30.))
                        .right(px(30.))
                        .bg(hsla(p.surface))
                        .p_4()
                        .display(Scale::X2, window)
                        .child(if g.paused { "PAUSED" } else { "GAME OVER" }),
                )
            });
        window_frame().child(power_on_in(
            "power",
            div()
                .id("ricochet-root")
                .track_focus(&self.focus)
                .flex()
                .flex_col()
                .size_full()
                .bg(hsla(p.bg))
                .text_color(hsla(p.fg))
                .body(text::BASE)
                .on_action(cx.listener(|this, _: &TogglePalette, window, cx| {
                    this.game.left = false;
                    this.game.right = false;
                    this.charging = false;
                    this.game.paused = true;
                    this.palette.update(cx, |p, cx| p.toggle(window, cx));
                }))
                .on_key_down(cx.listener(|this, ev: &gpui::KeyDownEvent, window, cx| {
                    if !this.focus.is_focused(window) {
                        return;
                    }
                    if !ev.is_held && ev.keystroke.key == "[" {
                        this.cycle_table(-1, window, cx);
                        return;
                    }
                    if !ev.is_held && ev.keystroke.key == "]" {
                        this.cycle_table(1, window, cx);
                        return;
                    }
                    this.key(&ev.keystroke.key, true, ev.is_held);
                    cx.notify();
                }))
                .on_key_up(cx.listener(|this, ev: &gpui::KeyUpEvent, window, cx| {
                    if !this.focus.is_focused(window) {
                        return;
                    }
                    this.key(&ev.keystroke.key, false, false);
                    cx.notify();
                }))
                .child(title_bar("Ricochet"))
                .child(
                    div()
                        .flex()
                        .flex_1()
                        .min_h_0()
                        .gap_4()
                        .p_4()
                        .child(board)
                        .child(side),
                )
                .child(
                    status_bar()
                        .left_live(if g.paused {
                            "PAUSED"
                        } else if g.tilt >= 1. {
                            "TILT / FLIPPERS DISABLED"
                        } else {
                            g.message
                        })
                        .right_live(format!(
                            "{} HITS / {} / Q + E",
                            g.hits,
                            if effects { "IMPACT ON" } else { "IMPACT OFF" }
                        )),
                )
                .child(self.palette.clone()),
        ))
    }
}
fn main() {
    if std::env::args().any(|a| a == "--benchmark-all") {
        benchmark::all();
        return;
    }
    if std::env::args().any(|a| a == "--snapshot-all") {
        benchmark::snapshot_all();
        return;
    }
    if std::env::args().any(|a| a == "--scene-snapshot") {
        benchmark::snapshot();
        return;
    }
    if std::env::args().any(|a| a == "--benchmark") {
        benchmark::run();
        return;
    }
    let args = std::env::args().collect::<Vec<_>>();
    if let Some(i) = args.iter().position(|a| a == "--profile") {
        let path = args
            .get(i + 1)
            .expect("--profile requires an output JSON path");
        profile::configure(path.into());
    }

    if std::env::args().any(|a| a == "--smoke") {
        benchmark::smoke_all();
        return;
    }
    if std::env::args().any(|a| a == "--version") {
        println!("Ricochet {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    gpui_platform::application().run(|cx: &mut App| {
        ferrite_design::init(Appearance::Dark, cx);
        theme::apply_env(cx);
        cx.bind_keys([KeyBinding::new("ctrl-shift-p", TogglePalette, None)]);
        let options = chrome::remembered_window_options(
            "ricochet",
            "Ricochet",
            size(px(1120.), px(860.)),
            cx,
        );
        cx.open_window(options, |window, cx| {
            chrome::square_corners(window);
            chrome::remember_window("ricochet", window, cx);
            chrome::power_off_on_close(window, cx);
            cx.new(|cx| Ricochet::new(window, cx))
        })
        .expect("open Ricochet");
        cx.activate(true);
    });
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    #[test]
    fn every_ferrite_scheme_resolves_to_its_own_board() {
        assert_eq!(SCHEMES.len(), board::ALL.len());
        for scheme in SCHEMES {
            assert_eq!(board::for_scheme(scheme.key).key, scheme.key);
        }
    }
}
