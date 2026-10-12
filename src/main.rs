#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod audio;
mod benchmark;
mod physics;
mod profile;
mod records;
mod scene;
mod table;
use ferrite_design::prelude::*;
use gpui::{
    App, AppContext as _, Context, Entity, FocusHandle, Focusable, IntoElement, KeyBinding, Render,
    Subscription, Task, Window, div, px, size,
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
    _tick: Task<()>,
    _appearance: Subscription,
}
impl Drop for Ricochet {
    fn drop(&mut self) {
        records::save(self.game.best);
    }
}
impl Focusable for Ricochet {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
impl Ricochet {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        let tick = cx.spawn(async move |this, cx| {
            let mut last = Instant::now();
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(16))
                    .await;
                let now = Instant::now();
                let dt = now.duration_since(last).as_secs_f32();
                last = now;
                if this
                    .update(cx, |this, cx| {
                        if this.game.paused && !this.demo {
                            return;
                        }
                        let active = this.demo
                            || this.game.live
                            || this.charging
                            || this.game.left
                            || this.game.right
                            || this.game.shaking > 0.
                            || (this.game.angles[0] - 0.42).abs() > 0.001
                            || (this.game.angles[1] - (std::f32::consts::PI - 0.42)).abs() > 0.001;
                        if !active {
                            return;
                        }
                        if this.demo {
                            this.game.paused = false;
                            if !this.game.live {
                                if this.game.balls == 0 {
                                    this.game.reset();
                                }
                                this.game.charge = 0.7;
                                this.game.launch();
                            }
                            this.game.left = (this.game.time * 2.3).sin() > 0.2;
                            this.game.right = (this.game.time * 1.9).cos() > 0.1;
                        }
                        let previous = (
                            this.game.hits,
                            this.game.score,
                            this.game.balls,
                            this.game.multiplier,
                        );
                        let was_live = this.game.live;
                        this.game.tick(dt, this.charging);
                        if this.game.balls < previous.2 {
                            this.audio.play(audio::Cue::Drain);
                        } else if this.game.multiplier > previous.3 {
                            this.audio.play(audio::Cue::Bank);
                        } else if this.game.hits > previous.0 {
                            this.audio.play(audio::Cue::Bumper);
                        } else if this.game.score > previous.1 {
                            this.audio.play(audio::Cue::Sling);
                        }
                        if was_live && !this.game.live {
                            records::save(this.game.best);
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
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
        ] {
            let weak = weak.clone();
            commands.push(command(label).group("Machine").on_run(move |window, cx| {
                let _ = weak.update(cx, |this, cx| {
                    match action {
                        0 => {
                            this.game.reset();
                            this.demo = false;
                        }
                        1 => this.game.paused = !this.game.paused,
                        2 => {
                            this.game.paused = false;
                            this.game.launch();
                        }
                        3 => this.effects = !this.effects,
                        4 => this.game.nudge(),
                        5 => this.audio.enabled = !this.audio.enabled,
                        _ => this.demo = !this.demo,
                    }
                    this.focus.focus(window, cx);
                    cx.notify();
                });
            }));
        }
        for scheme in SCHEMES {
            commands.push(
                command(format!("Scheme: {}", scheme.name))
                    .group("Appearance")
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
                best: records::load(),
                ..physics::Game::default()
            },
            audio: audio::Audio {
                enabled: cfg!(windows) && !profile::enabled(),
                ..audio::Audio::default()
            },
            palette,
            focus,
            charging: false,
            effects: true,
            demo: std::env::args().any(|a| a == "--demo") || profile::enabled(),
            _tick: tick,
            _appearance: theme::follow_system(window),
        }
    }
    fn key(&mut self, key: &str, down: bool, held: bool) {
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
                    self.charging = false;
                    let was_live = self.game.live;
                    self.game.launch();
                    if !was_live && self.game.live {
                        self.audio.play(audio::Cue::Launch);
                    }
                }
            }
            "p" if down && !held => {
                self.game.paused = !self.game.paused;
                self.game.left = false;
                self.game.right = false;
                self.charging = false;
            }
            "r" if down && !held => {
                self.game.reset();
                self.charging = false;
            }
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
        let side = div()
            .id("machine-hud")
            .w(px(280.))
            .flex_shrink_0()
            .min_h_0()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap_2()
            .p_2()
            .child(div().display(Scale::X2, window).child("RICOCHET"))
            .child(
                div()
                    .body(text::SM)
                    .text_color(hsla(p.fg_dim))
                    .child("FERROUS / SERIES 02"),
            )
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
                "BALL {} / 3       {}X BANK",
                (4 - g.balls).min(3),
                g.multiplier
            )))
            .child(
                div()
                    .display(Scale::X1, window)
                    .text_color(hsla(if g.combo > 1 { p.accent_text } else { p.fg_dim }))
                    .child(format!("{}X COMBO     {} HITS", g.combo.max(1), g.hits)),
            )
            .child(ascii_rule(Some("CIRCUIT BREAKER")))
            .child(lights(g.lamps))
            .child(
                div()
                    .body(text::SM)
                    .text_color(hsla(p.fg_dim))
                    .child("Bumpers: 100. Complete the bank for +500 and a higher multiplier."),
            )
            .child(ascii_rule(Some("RETURN LANES")))
            .child(lights(g.rollovers))
            .child(
                div()
                    .body(text::SM)
                    .text_color(hsla(p.fg_dim))
                    .child("Lanes: 50. Light all three for +250. Chain bumper hits for combos."),
            )
            .child(ascii_rule(Some("MACHINE")))
            .child(ascii_gauge(g.charge).label("PULL").cells(12))
            .child(ascii_gauge(g.tilt.clamp(0., 1.)).label("TILT").cells(12))
            .child(
                div()
                    .display(Scale::X1, window)
                    .text_color(hsla(if g.tilt >= 1. {
                        p.danger
                    } else if g.save_left > 0. {
                        p.accent_text
                    } else {
                        p.fg_dim
                    }))
                    .child(if g.tilt >= 1. {
                        "TILT / FLIPPERS LOCKED".into()
                    } else if g.save_left > 0. {
                        format!("BALL SAVE  {:02}", g.save_left.ceil() as u32)
                    } else if self.demo {
                        "ATTRACT MODE".into()
                    } else if g.paused {
                        "PAUSED / P TO RESUME".into()
                    } else {
                        "Q + E / KEEP IT ALIVE".into()
                    }),
            )
            .child(div().flex_1())
            .child(div().body(text::SM).text_color(hsla(p.fg_dim)).children([
                "Q / E    Hold flippers",
                "SPACE    Pull / release plunger",
                "N  Nudge   P  Pause   F  Effects",
                "R  Restart   CTRL SHIFT P  Menu",
                "M  Sound (Windows)",
            ]))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        Button::new("new-game")
                            .label("New game")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.game.reset();
                                this.charging = false;
                                this.demo = false;
                                this.focus.focus(window, cx);
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("pause")
                            .label(if g.paused { "Resume" } else { "Pause" })
                            .on_click(cx.listener(|this, _, window, cx| {
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
                        .left_live(if g.paused { "PAUSED" } else { g.message })
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
        let mut g = physics::Game {
            charge: 1.,
            ..physics::Game::default()
        };
        g.launch();
        for i in 0..2400 {
            g.left = i % 60 < 30;
            g.right = i % 73 < 35;
            g.tick(1. / 120., false);
            assert!(g.ball.x.is_finite() && g.ball.y.is_finite());
        }
        assert!(g.hits > 0);
        println!("Physics smoke passed: {} hits, score {}", g.hits, g.score);
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
