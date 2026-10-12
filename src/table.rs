use crate::physics::{BUMPERS, Game, V, WALLS};
use ferrite_design::prelude::*;
use gpui::{Bounds, IntoElement, Pixels, Window, canvas, point, px, size};
fn distance(p: V, a: V, b: V) -> f32 {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.dot(ab)).clamp(0., 1.);
    (p - a - ab * t).len()
}
const BAYER: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
pub fn table(g: Game, effects: bool, cx: &gpui::App) -> impl IntoElement {
    let p = palette(cx);
    let colors = [
        hsla(p.sunken),
        hsla(p.surface),
        hsla(p.line),
        hsla(p.fg_faint),
        hsla(p.fg_dim),
        hsla(p.fg),
        hsla(p.accent),
        hsla(p.accent_dim),
    ];
    canvas(
        |_, _, _| {},
        move |bounds: Bounds<Pixels>, _, window: &mut Window, _| {
            let scale =
                (f32::from(bounds.size.width) / 244.).min(f32::from(bounds.size.height) / 380.);
            let sf = window.scale_factor();
            let scale = (scale * sf).floor().max(1.) / sf;
            let ox = bounds.left() + px((f32::from(bounds.size.width) - 244. * scale) / 2.);
            let oy = bounds.top() + px((f32::from(bounds.size.height) - 380. * scale) / 2.);
            let shake = if effects {
                (g.time * 95.).sin().signum() * g.shaking * 2.
            } else {
                0.
            };
            for y in 0..380 {
                let mut start = 0;
                let mut last = 0;
                for x in 0..=244 {
                    let color = if x == 244 {
                        255
                    } else {
                        let q = V::new(x as f32 - shake, y as f32);
                        let threshold = BAYER[y % 4][x % 4];
                        let mut c = if x % 24 == 0 && y % 24 == 0 { 2 } else { 0 };
                        // Dithered outer apron and stamped ribs, never a gradient.
                        if !(18..=224).contains(&x) || y < 18 || y > 360 {
                            c = if threshold < 5 { 2 } else { 1 };
                        }
                        if (32..190).contains(&x) && (32..205).contains(&y) && threshold == 0 {
                            c = 1;
                        }
                        for (a, b) in WALLS {
                            let d = distance(q, a, b);
                            if d < 2. {
                                c = 5;
                            } else if d < 6. {
                                c = if threshold < 8 { 3 } else { 1 };
                            }
                        }
                        for (i, b) in BUMPERS.iter().enumerate() {
                            let d = (q - *b).len();
                            if d < 14. {
                                c = if d > 11. {
                                    5
                                } else if threshold < 7 {
                                    3
                                } else {
                                    1
                                };
                                if g.lamps[i] && d < 6. {
                                    c = 6;
                                }
                                if effects
                                    && g.flash[i] > 0.
                                    && threshold < (g.flash[i] * 16.) as u8
                                {
                                    c = 6;
                                }
                            }
                            if effects && g.flash[i] > 0. {
                                let r = 19. + (1. - g.flash[i]) * 27.;
                                if (d - r).abs() < 1. && threshold < 8 {
                                    c = 6;
                                }
                            }
                        }
                        for i in 0..2 {
                            let (a, b) = g.flipper(i);
                            let d = distance(q, a, b);
                            if d < 3. {
                                c = if (i == 0 && g.left) || (i == 1 && g.right) {
                                    6
                                } else {
                                    5
                                };
                            } else if d < 5. {
                                c = 3;
                            }
                        }
                        // Sling plates and three rollover gates.
                        for (a, b) in [
                            (V::new(44., 220.), V::new(70., 247.)),
                            (V::new(179., 220.), V::new(155., 247.)),
                        ] {
                            if distance(q, a, b) < 7. && threshold < 9 {
                                c = 4;
                            }
                        }
                        for i in 0..3 {
                            if (q.x - (65. + i as f32 * 42.)).abs() < 10. && (42..57).contains(&y) {
                                c = if x % 3 == 0 { 4 } else { 1 };
                            }
                        }
                        if effects {
                            for (i, t) in g.trail.iter().enumerate() {
                                if (q - *t).len() < 2. && threshold < (i + 1) as u8 {
                                    c = 3;
                                }
                            }
                        }
                        let d = (q - g.ball).len();
                        if d < 4. {
                            c = if q.x < g.ball.x && q.y < g.ball.y {
                                5
                            } else {
                                6
                            };
                        }
                        if x > 206 && x < 218 && y > 341 && y < 355 {
                            c = if (y as f32 - 341.) < g.charge * 14. {
                                6
                            } else {
                                3
                            };
                        }
                        c
                    };
                    if color != last {
                        if last != 0 && x > start {
                            window.paint_quad(gpui::fill(
                                Bounds::new(
                                    point(ox + px(start as f32 * scale), oy + px(y as f32 * scale)),
                                    size(px((x - start) as f32 * scale), px(scale)),
                                ),
                                colors[last as usize],
                            ));
                        }
                        start = x;
                        last = color;
                    }
                }
            }
        },
    )
    .size_full()
}
