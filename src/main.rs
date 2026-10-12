#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod physics;
mod records;
mod table;
use ferrite_design::prelude::*;
use gpui::{
    App, AppContext as _, Context, Entity, FocusHandle, Focusable, IntoElement, KeyBinding, Render,
    Subscription, Task, Window, div, px, size,
};
use std::time::{Duration, Instant};
struct Ricochet {
    game: physics::Game,
    palette: Entity<CommandPalette>,
    focus: FocusHandle,
    charging: bool,
    effects: bool,
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
                        let was_live = this.game.live;
                        this.game.tick(dt, this.charging);
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
        ] {
            let weak = weak.clone();
            commands.push(command(label).group("Machine").on_run(move |window, cx| {
                let _ = weak.update(cx, |this, cx| {
                    match action {
                        0 => this.game.reset(),
                        1 => this.game.paused = !this.game.paused,
                        2 => this.game.launch(),
                        _ => this.effects = !this.effects,
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
            palette,
            focus,
            charging: false,
            effects: true,
            _tick: tick,
            _appearance: theme::follow_system(window),
        }
    }
    fn key(&mut self, key: &str, down: bool, held: bool) {
        match key {
            "q" => self.game.left = down,
            "e" => self.game.right = down,
            "space" => {
                if down {
                    self.charging = true;
                } else {
                    self.charging = false;
                    self.game.launch();
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
            "f" if down && !held => self.effects = !self.effects,
            _ => {}
        }
    }
}
impl Render for Ricochet {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !window.is_window_active() {
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
        let readout = |label: &str, value: String| {
            ascii_box()
                .title(label.to_owned())
                .child(div().display(Scale::X2, window).child(value))
        };
        let side =
            div()
                .w(px(268.))
                .flex_shrink_0()
                .flex()
                .flex_col()
                .gap_4()
                .child(div().display(Scale::X2, window).child("RICOCHET"))
                .child(
                    div()
                        .body(text::SM)
                        .text_color(hsla(p.fg_dim))
                        .child("FERROUS PINBALL / MODEL 01"),
                )
                .child(readout("SCORE", format!("{:08}", g.score)))
                .child(readout("HIGH WATER", format!("{:08}", g.best)))
                .child(
                    ascii_box().title("CIRCUIT").child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .display(Scale::X1, window)
                                    .child(format!("BALLS {}  /  {}X", g.balls, g.multiplier)),
                            )
                            .child(ascii_gauge(g.charge).label("PLUNGER").cells(18))
                            .child(div().body(text::SM).child(
                                "Light all 3 bumpers: +500 and multiplier up. Slings: +25.",
                            )),
                    ),
                )
                .child(ascii_box().title("OPERATOR").child(
                    div().flex().flex_col().gap_2().body(text::SM).children([
                        "Q / E    HOLD FLIPPERS",
                        "SPACE    CHARGE / RELEASE",
                        "N        NUDGE (WATCH TILT)",
                        "P        PAUSE",
                        "R        NEW GAME",
                        "F        IMPACT EFFECTS",
                        "CTRL SHIFT P   COMMANDS",
                    ]),
                ))
                .child(
                    Button::new("new-game")
                        .label("New game")
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.game.reset();
                            this.focus.focus(window, cx);
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("pause")
                        .label(if g.paused { "Resume" } else { "Pause" })
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.game.paused = !this.game.paused;
                            this.focus.focus(window, cx);
                            cx.notify();
                        })),
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
            size(px(1000.), px(820.)),
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
