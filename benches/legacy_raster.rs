// Frozen native raster workload from 0502877; excludes upscale and GPU upload.
use crate::physics::{BUMPERS, Game, V, WALLS};
fn distance(p: V, a: V, b: V) -> f32 {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.dot(ab)).clamp(0., 1.);
    (p - a - ab * t).len()
}
const BAYER: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
// Three-by-five stamped cabinet type, rasterised on the same pixel grid.
fn stamp(x: usize, y: usize) -> bool {
    if !(72..136).contains(&x) || !(193..203).contains(&y) {
        return false;
    }
    let col = (x - 72) / 2;
    let row = (y - 193) / 2;
    let glyphs: [[u8; 5]; 8] = [
        [6, 5, 6, 5, 5],
        [7, 2, 2, 2, 7],
        [7, 4, 4, 4, 7],
        [7, 5, 5, 5, 7],
        [7, 4, 4, 4, 7],
        [5, 5, 7, 5, 5],
        [7, 4, 6, 4, 7],
        [7, 2, 2, 2, 2],
    ];
    let local = col % 4;
    local < 3 && (glyphs[col / 4][row] & (1 << (2 - local))) != 0
}

pub fn raster(g: &Game, effects: bool) -> Vec<u8> {
    let colors = [
        [1u8; 4], [2; 4], [3; 4], [4; 4], [5; 4], [6; 4], [7; 4], [8; 4],
    ];
    let shake = if effects {
        (g.time * 95.).sin().signum() * g.shaking * 2.
    } else {
        0.
    };
    let mut pixels = Vec::with_capacity(244 * 380 * 4);
    for y in 0..380 {
        for x in 0..244 {
            let color = {
                let q = V::new(x as f32 - shake, y as f32 - shake * 0.5);
                let threshold = BAYER[y % 4][x % 4];
                let mut c = if x % 24 == 0 && y % 24 == 0 { 2 } else { 0 };
                // Dithered outer apron and stamped ribs, never a gradient.
                if !(18..=224).contains(&x) || !(18..=360).contains(&y) {
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
                        if effects && g.flash[i] > 0. && threshold < (g.flash[i] * 16.) as u8 {
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
                if stamp(x, y) {
                    c = 4;
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
            pixels.extend_from_slice(&colors[color]);
        }
    }
    pixels
}
