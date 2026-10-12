//! Perspective geometry shared by the live GPU canvas and the CPU benchmark.
use crate::board::{Board, Rail};
use crate::physics::{Game, V};
use std::f32::consts::{PI, TAU};
#[path = "scenery.rs"]
mod scenery;
#[derive(Clone, Copy, Debug, Default)]
pub struct P3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl P3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    fn dot(self, b: Self) -> f32 {
        self.x * b.x + self.y * b.y + self.z * b.z
    }
    fn sub(self, b: Self) -> Self {
        Self::new(self.x - b.x, self.y - b.y, self.z - b.z)
    }
    fn cross(self, b: Self) -> Self {
        Self::new(
            self.y * b.z - self.z * b.y,
            self.z * b.x - self.x * b.z,
            self.x * b.y - self.y * b.x,
        )
    }
    fn unit(self) -> Self {
        let n = self.dot(self).sqrt();
        Self::new(self.x / n, self.y / n, self.z / n)
    }
}
#[derive(Clone, Debug)]
pub struct Face {
    pub points: Vec<P3>,
    pub ink: usize,
    pub floor: bool,
}
#[derive(Clone, Debug)]
pub struct Projected {
    pub points: Vec<[f32; 2]>,
    pub depth: f32,
    pub ink: usize,
    pub floor: bool,
}
#[derive(Clone, Copy)]
pub struct Camera {
    eye: P3,
    forward: P3,
    right: P3,
    up: P3,
    pub scale: f32,
    pub ox: f32,
    pub oy: f32,
}
impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        let eye = P3::new(122., 660., 640.);
        let forward = P3::new(122., 175., 0.).sub(eye).unit();
        let right = P3::new(0., 0., 1.).cross(forward).unit();
        let up = forward.cross(right).unit();
        let mut c = Self {
            eye,
            forward,
            right,
            up,
            scale: 1.,
            ox: 0.,
            oy: 0.,
        };
        let corners = [
            P3::new(-8., -13., 64.),
            P3::new(252., -13., 64.),
            P3::new(-8., 393., -24.),
            P3::new(252., 393., -24.),
        ];
        let points = corners.map(|p| c.project(p).0);
        let mut lo = [f32::MAX; 2];
        let mut hi = [f32::MIN; 2];
        for p in points {
            for i in 0..2 {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
        }
        c.scale =
            ((width - 24.).max(1.) / (hi[0] - lo[0])).min((height - 24.).max(1.) / (hi[1] - lo[1]));
        c.ox = (width - (hi[0] - lo[0]) * c.scale) / 2. - lo[0] * c.scale;
        c.oy = (height - (hi[1] - lo[1]) * c.scale) / 2. - lo[1] * c.scale;
        c
    }
    pub fn project(&self, p: P3) -> ([f32; 2], f32) {
        let r = p.sub(self.eye);
        let depth = r.dot(self.forward);
        let k = 760. / depth;
        (
            [
                self.ox + r.dot(self.right) * k * self.scale,
                self.oy - r.dot(self.up) * k * self.scale,
            ],
            depth,
        )
    }
    pub fn face(&self, f: &Face) -> Projected {
        let mut depth = 0.;
        let points = f
            .points
            .iter()
            .map(|p| {
                let (x, d) = self.project(*p);
                depth += d;
                x
            })
            .collect();
        Projected {
            points,
            depth: depth / f.points.len() as f32,
            ink: f.ink,
            floor: f.floor,
        }
    }
}
fn face(out: &mut Vec<Face>, points: impl Into<Vec<P3>>, ink: usize, floor: bool) {
    out.push(Face {
        points: points.into(),
        ink,
        floor,
    });
}
fn plate(out: &mut Vec<Face>, xy: &[V], z: f32, ink: usize) {
    face(
        out,
        xy.iter().map(|v| P3::new(v.x, v.y, z)).collect::<Vec<_>>(),
        ink,
        true,
    );
}
fn rect(out: &mut Vec<Face>, x: f32, y: f32, w: f32, h: f32, z: f32, ink: usize) {
    plate(
        out,
        &[
            V::new(x, y),
            V::new(x + w, y),
            V::new(x + w, y + h),
            V::new(x, y + h),
        ],
        z,
        ink,
    );
}
fn prism(out: &mut Vec<Face>, outline: &[V], z: f32, height: f32, top: usize) {
    for i in 0..outline.len() {
        let a = outline[i];
        let b = outline[(i + 1) % outline.len()];
        let edge = b - a;
        let shade = if edge.x > 0. {
            5
        } else if edge.y > 0. {
            4
        } else {
            3
        };
        face(
            out,
            vec![
                P3::new(a.x, a.y, z),
                P3::new(b.x, b.y, z),
                P3::new(b.x, b.y, z + height),
                P3::new(a.x, a.y, z + height),
            ],
            shade,
            false,
        );
    }
    face(
        out,
        outline
            .iter()
            .map(|v| P3::new(v.x, v.y, z + height))
            .collect::<Vec<_>>(),
        top,
        false,
    );
}
fn strip(out: &mut Vec<Face>, a: V, b: V, r: f32, z: f32, height: f32, ink: usize) {
    let d = b - a;
    let n = V::new(-d.y, d.x) * (r / d.len());
    if height <= 0.0001 {
        let outline = [a + n, b + n, b - n, a - n];
        face(
            out,
            outline
                .iter()
                .map(|v| P3::new(v.x, v.y, z))
                .collect::<Vec<_>>(),
            ink,
            z <= 0.5,
        );
    } else {
        prism(out, &[a + n, b + n, b - n, a - n], z, height, ink);
    }
}
fn ring(out: &mut Vec<Face>, center: V, r: f32, width: f32, z: f32, ink: usize) {
    for i in 0..24 {
        let angle = i as f32 * TAU / 24.;
        let next = (i + 1) as f32 * TAU / 24.;
        let p =
            |radius: f32, t: f32| V::new(center.x + radius * t.cos(), center.y + radius * t.sin());
        plate(
            out,
            &[
                p(r, angle),
                p(r, next),
                p(r - width, next),
                p(r - width, angle),
            ],
            z,
            ink,
        );
    }
}
fn cylinder(out: &mut Vec<Face>, center: V, r: f32, z: f32, h: f32, top: usize) {
    let outline = (0..16)
        .map(|i| {
            let a = i as f32 * TAU / 16.;
            V::new(center.x + r * a.cos(), center.y + r * a.sin())
        })
        .collect::<Vec<_>>();
    prism(out, &outline, z, h, top);
}
fn glyph(out: &mut Vec<Face>, x: f32, y: f32, s: f32, ch: char, ink: usize) {
    let rows = match ch {
        'G' => [7, 4, 5, 5, 7],
        'J' => [1, 1, 1, 5, 7],
        'K' => [5, 5, 6, 5, 5],
        'W' => [5, 5, 7, 7, 5],
        'Y' => [5, 5, 2, 2, 2],
        'Z' => [7, 1, 2, 4, 7],
        '6' => [7, 4, 7, 5, 7],
        '7' => [7, 1, 2, 2, 2],
        '8' => [7, 5, 7, 5, 7],
        '9' => [7, 5, 7, 1, 7],
        '/' => [1, 1, 2, 4, 4],
        'Q' => [7, 5, 5, 7, 1],
        '+' => [0, 2, 7, 2, 0],
        'R' => [6, 5, 6, 5, 5],
        'I' => [7, 2, 2, 2, 7],
        'C' => [7, 4, 4, 4, 7],
        'O' => [7, 5, 5, 5, 7],
        'H' => [5, 5, 7, 5, 5],
        'E' => [7, 4, 6, 4, 7],
        'T' => [7, 2, 2, 2, 2],
        'F' => [7, 4, 6, 4, 4],
        'U' => [5, 5, 5, 5, 7],
        'S' => [7, 4, 7, 1, 7],
        'N' => [5, 7, 7, 7, 5],
        'A' => [2, 5, 7, 5, 5],
        'L' => [4, 4, 4, 4, 7],
        'B' => [6, 5, 6, 5, 6],
        'M' => [5, 7, 7, 5, 5],
        'D' => [6, 5, 5, 5, 6],
        'P' => [6, 5, 6, 4, 4],
        'V' => [5, 5, 5, 5, 2],
        '0' => [7, 5, 5, 5, 7],
        '1' => [2, 6, 2, 2, 7],
        '2' => [6, 1, 2, 4, 7],
        '3' => [6, 1, 6, 1, 6],
        '4' => [5, 5, 7, 1, 1],
        '5' => [7, 4, 6, 1, 6],
        'X' => [5, 5, 2, 5, 5],
        _ => [0; 5],
    };
    for (row, bits) in rows.into_iter().enumerate() {
        for col in 0..3 {
            if bits & (1 << (2 - col)) != 0 {
                rect(
                    out,
                    x + col as f32 * s,
                    y + row as f32 * s,
                    s * 0.82,
                    s * 0.82,
                    0.3,
                    ink,
                );
            }
        }
    }
}
fn lettering(out: &mut Vec<Face>, x: f32, y: f32, s: f32, text: &str, ink: usize) {
    for (i, ch) in text.chars().enumerate() {
        glyph(out, x + i as f32 * s * 4., y, s, ch, ink);
    }
}
fn rail_mesh(out: &mut Vec<Face>, rail: &Rail, deck: bool) {
    for i in 0..rail.points.len() - 1 {
        let a = rail.points[i];
        let b = rail.points[i + 1];
        let d = b - a;
        let n = V::new(-d.y, d.x) * (1. / d.len());
        let z = rail.height(rail.distances[i]);
        if deck {
            strip(out, a, b, 4., z, 0.7, 3);
        }
        for side in [-5., 5.] {
            strip(
                out,
                a + n * side,
                b + n * side,
                0.9,
                z + if deck { 0.7 } else { 0. },
                3.,
                7,
            );
        }
    }
    if deck {
        for i in (8..rail.points.len() - 8).step_by(12) {
            let h = rail.height(rail.distances[i]);
            if h > 6. {
                cylinder(out, rail.points[i], 2.2, 0., h, 5);
            }
        }
    }
}
fn etching(out: &mut Vec<Face>, board: &Board) {
    match board.design {
        0 => {
            for r in [29., 39.] {
                ring(out, board.scoop, r, 0.7, 0.3, 4);
            }
            for i in 0..16 {
                let a = i as f32 * TAU / 16.;
                let c = board.scoop + V::new(a.cos(), a.sin()) * 35.;
                rect(out, c.x - 1., c.y - 1., 2., 2., 0.4, 5);
            }
        }
        1 => {
            for i in 0..7 {
                let y = 80. + i as f32 * 24.;
                rect(out, 48., y, 132., 0.7, 0.3, 4);
            }
            for x in [66., 90., 114., 138., 162.] {
                rect(out, x, 68., 0.7, 151., 0.3, 4);
            }
        }
        2 => {
            for i in 0..4 {
                let y = 72. + i as f32 * 42.;
                let poly = (0..6)
                    .map(|j| {
                        let a = j as f32 * TAU / 6.;
                        V::new(112. + 44. * a.cos(), y + 23. * a.sin())
                    })
                    .collect::<Vec<_>>();
                for j in 0..6 {
                    strip(out, poly[j], poly[(j + 1) % 6], 0.6, 0.3, 0., 4);
                }
            }
        }
        3 => {
            for row in 0..5 {
                for i in 0..24 {
                    let x = 48. + i as f32 * 5.4;
                    let next = x + 5.4;
                    let y = 159. + row as f32 * 8.;
                    strip(
                        out,
                        V::new(x, y + (x * 0.06).sin() * 10.),
                        V::new(next, y + (next * 0.06).sin() * 10.),
                        0.45,
                        0.3,
                        0.,
                        4,
                    );
                }
            }
        }
        4 => {
            for i in 0..6 {
                let x = 55. + i as f32 * 20.;
                strip(out, V::new(x, 70.), V::new(112., 220.), 0.6, 0.3, 0., 4);
                rect(out, x, 67., 5., 2., 0.3, 5);
            }
        }
        5 => {
            for r in [20., 32., 44.] {
                ring(out, V::new(119., 173.), r, 0.7, 0.3, 4);
            }
            for i in 0..6 {
                let y = 106. + i as f32 * 24.;
                strip(out, V::new(58., y), V::new(164., y + 25.), 0.45, 0.3, 0., 4);
            }
        }
        6 => {
            for x in (45..190).step_by(18) {
                rect(out, x as f32, 60., 0.4, 169., 0.3, 4);
            }
            for y in (66..224).step_by(18) {
                rect(out, 40., y as f32, 152., 0.4, 0.3, 4);
            }
            ring(out, board.scoop, 24., 0.6, 0.3, 5);
        }
        7 => {
            for r in [20., 31., 42., 53.] {
                ring(out, V::new(116., 165.), r, 0.6, 0.3, 4);
            }
            for i in 0..12 {
                let a = i as f32 * TAU / 12.;
                strip(
                    out,
                    V::new(116., 165.) + V::new(a.cos(), a.sin()) * 45.,
                    V::new(116., 165.) + V::new(a.cos(), a.sin()) * 56.,
                    0.6,
                    0.3,
                    0.,
                    5,
                );
            }
        }
        8 => {
            for side in [-1., 1.] {
                for i in 0..5 {
                    let y = 92. + i as f32 * 24.;
                    strip(
                        out,
                        V::new(112., y),
                        V::new(112. + side * 48., y + 38.),
                        0.5,
                        0.3,
                        0.,
                        4,
                    );
                }
            }
        }
        _ => {
            for i in 0..10 {
                let x = 52. + i as f32 * 11.;
                strip(out, V::new(x, 177.), V::new(x + 22., 209.), 0.7, 0.3, 0., 4);
            }
            plate(
                out,
                &[
                    V::new(62., 87.),
                    V::new(143., 54.),
                    V::new(132., 81.),
                    V::new(177., 72.),
                ],
                0.3,
                4,
            );
        }
    }
}
pub fn static_table(board: &Board) -> Vec<Face> {
    let mut m = Vec::with_capacity(1400);
    // Cabinet: a solid body below the playfield, with a machined lip.
    prism(
        &mut m,
        &[
            V::new(0., 0.),
            V::new(244., 0.),
            V::new(244., 380.),
            V::new(0., 380.),
        ],
        -22.,
        22.,
        2,
    );
    m[4].floor = true;
    rect(&mut m, 18., 18., 204., 338., 0.1, 1);
    for (a, b) in [
        (V::new(2., 2.), V::new(242., 2.)),
        (V::new(2., 2.), V::new(2., 376.)),
        (V::new(242., 2.), V::new(242., 376.)),
        (V::new(2., 376.), V::new(242., 376.)),
    ] {
        strip(&mut m, a, b, 3., 0., 10., 5);
    }
    // Etched traces and crosshatched steel are static geometry, cached once.
    for y in (26..350).step_by(12) {
        for x in (30..194).step_by(12) {
            rect(&mut m, x as f32, y as f32, 0.65, 0.65, 0.25, 4);
        }
    }
    for side in [0., 1.] {
        let x = if side == 0. { 33. } else { 190. };
        for lane in 0..4 {
            let dx = if side == 0. { 1. } else { -1. };
            let a = V::new(x + dx * lane as f32 * 3., 70.);
            let b = V::new(x + dx * (24. + lane as f32 * 3.), 195.);
            strip(&mut m, a, b, 0.35, 0.1, 0., 4);
        }
    }
    etching(&mut m, board);
    lettering(&mut m, 65., 207., 3., "RICOCHET", 6);
    lettering(
        &mut m,
        112. - board.title.len() as f32 * 3.,
        227.,
        1.5,
        board.title,
        4,
    );
    lettering(&mut m, 47., 338., 1.4, "Q", 7);
    lettering(&mut m, 164., 338., 1.4, "E", 7);
    lettering(&mut m, 91., 285., 1.1, "BALL SAVE", 5);
    // Rails and raised mechanisms follow the same collision geometry as physics.
    for &(a, b) in board.walls {
        strip(&mut m, a, b, 2.2, 0., 7., 7);
        strip(&mut m, a, b, 0.65, 7.1, 0., 8);
    }
    for &c in board.bumpers {
        ring(&mut m, c, 22., 1.2, 0.4, 5);
        cylinder(&mut m, c, 15., 0., 6., 5);
        cylinder(&mut m, c, 11.5, 6., 10., 7);
        cylinder(&mut m, c, 14., 16., 3., 8);
        cylinder(&mut m, c, 10.5, 19., 1., 3);
        ring(&mut m, c, 8., 1.2, 20.2, 7);
    }
    for (a, b, tip) in [
        (V::new(43., 220.), V::new(73., 257.), V::new(43., 263.)),
        (V::new(181., 220.), V::new(153., 257.), V::new(180., 263.)),
    ] {
        prism(&mut m, &[a, b, tip], 0., 5., 3);
        strip(&mut m, a, b, 1.6, 5., 2., 7);
        let middle = (a + b) * 0.5;
        lettering(&mut m, middle.x - 7., middle.y + 3., 1., "25", 6);
    }
    // Rollover lanes, miniature posts, fasteners and return-lane markings.
    for i in 0..3 {
        let x = board.lanes[i].x;
        let y = board.lanes[i].y;
        for dx in [-12., 12.] {
            strip(
                &mut m,
                V::new(x + dx, y - 16.),
                V::new(x + dx, y + 6.),
                1.,
                0.,
                5.,
                6,
            );
        }
        lettering(&mut m, x - 2., y - 11., 1.6, &(i + 1).to_string(), 7);
    }
    for (x, y) in [
        (30., 75.),
        (194., 75.),
        (36., 207.),
        (189., 207.),
        (75., 271.),
        (151., 271.),
        (44., 314.),
        (180., 314.),
    ] {
        cylinder(&mut m, V::new(x, y), 3., 0., 6., 7);
        cylinder(&mut m, V::new(x, y), 1.2, 6., 0.5, 8);
    }
    for y in [135., 165., 195., 225., 255., 285.] {
        plate(
            &mut m,
            &[
                V::new(208., y + 5.),
                V::new(212., y),
                V::new(216., y + 5.),
                V::new(212., y + 3.),
            ],
            0.4,
            5,
        );
    }
    for y in (24..365).step_by(13) {
        for x in [7., 232.] {
            rect(&mut m, x, y as f32, 4., 1., 10., 4);
        }
    }
    rail_mesh(&mut m, board.orbit, false);
    rail_mesh(&mut m, board.ramp, true);
    for t in board.targets {
        prism(
            &mut m,
            &[
                t.center + V::new(-6., -3.),
                t.center + V::new(6., -3.),
                t.center + V::new(6., 3.),
                t.center + V::new(-6., 3.),
            ],
            0.,
            1.,
            5,
        );
    }
    cylinder(&mut m, board.scoop, 12., 0., 1.5, 5);
    cylinder(&mut m, board.scoop, 8.5, 1.6, 0.5, 3);
    // Front apron plate and plunger assembly.
    rect(&mut m, 44., 350., 151., 16., 0.3, 3);
    lettering(
        &mut m,
        65.,
        354.,
        1.3,
        &format!("TABLE {:02}", board.design + 1),
        7,
    );
    strip(
        &mut m,
        V::new(212., 348.),
        V::new(212., 371.),
        2.2,
        0.,
        5.,
        6,
    );
    cylinder(&mut m, V::new(212., 373.), 5., 0., 4., 7);
    scenery::decorate(&mut m, board);
    m
}
pub fn dynamic_table(g: &Game, effects: bool) -> Vec<Face> {
    let mut m = Vec::with_capacity(400);
    scenery::progress_lights(&mut m, g);
    let (a, b) = g.board.ramp_gate;
    let end = if g.ramp_open {
        V::new(a.x, a.y - 17.)
    } else {
        b
    };
    strip(
        &mut m,
        a,
        end,
        1.5,
        0.,
        8.,
        if g.ramp_open { 10 } else { 8 },
    );
    for i in 0..3 {
        let t = g.board.targets[i];
        let height = if g.drops[i] {
            0.8 + if effects {
                g.target_flash[i].max(0.) * 7.2
            } else {
                0.
            }
        } else {
            8.
        };
        let next =
            !g.ramp_open && g.target_progress < 3 && g.board.order[g.target_progress as usize] == i;
        let ink = if effects && g.target_flash[i] < 0. {
            12
        } else if next || g.target_flash[i] > 0. {
            10
        } else {
            7
        };
        prism(
            &mut m,
            &[
                t.center + V::new(-5., -2.),
                t.center + V::new(5., -2.),
                t.center + V::new(5., 2.),
                t.center + V::new(-5., 2.),
            ],
            1.,
            height,
            if g.drops[i] { 5 } else { ink },
        );
        if height > 5. {
            let mut numeral = Vec::new();
            glyph(&mut numeral, -1.4, 0., 0.95, char::from(b'1' + i as u8), 8);
            for mut f in numeral {
                for p in &mut f.points {
                    let py = p.y;
                    p.x += t.center.x;
                    p.y = t.center.y + 2.1;
                    p.z = height - py;
                }
                f.floor = false;
                m.push(f);
            }
        }
    }
    let (a, b) = g.board.gate;
    let end = if g.mission >= 1 {
        V::new(a.x, a.y - 17.)
    } else {
        b
    };
    strip(
        &mut m,
        a,
        end,
        1.5,
        0.,
        8.,
        if g.mission >= 1 { 10 } else { 8 },
    );
    cylinder(
        &mut m,
        g.board.scoop,
        7.,
        2.2,
        0.2,
        if g.mission >= 2 || g.jackpot_ready {
            0
        } else {
            5
        },
    );
    if g.mission >= 2 || g.jackpot_ready {
        ring(&mut m, g.board.scoop, 13., 1.5, 0.5, 10);
    }
    if g.jackpot_ready {
        lettering(
            &mut m,
            g.board.scoop.x - 14.,
            g.board.scoop.y + 15.,
            1.,
            "JACKPOT",
            10,
        );
    }
    if g.overdrive > 0. {
        ring(&mut m, g.board.scoop, 27., 1., 0.5, 10);
    }

    for i in 0..3 {
        cylinder(
            &mut m,
            g.board.lanes[i] + V::new(0., 10.),
            2.5,
            0.,
            1.,
            if g.rollovers[i] || g.lane_flash[i] > 0. {
                10
            } else {
                5
            },
        );
    }
    if g.save_left > 0. {
        lettering(&mut m, 91., 285., 1.1, "BALL SAVE", 10);
    }
    if g.combo > 1 {
        lettering(&mut m, 97., 245., 1.3, &format!("{}X", g.combo), 10);
    }
    for (i, c) in g.board.bumpers.iter().enumerate() {
        cylinder(
            &mut m,
            *c,
            6.,
            20.4,
            1.,
            if g.lamps[i] || g.flash[i] > 0. { 10 } else { 6 },
        );
        if effects && g.flash[i] > 0. {
            ring(&mut m, *c, 19. + (1. - g.flash[i]) * 20., 1., 0.5, 10);
        }
    }
    for i in 0..2 {
        let (a, b) = g.flipper(i);
        let d = (b - a) * (1. / 37.);
        let n = V::new(-d.y, d.x);
        let mut outline = Vec::with_capacity(16);
        for j in 0..8 {
            let t = -PI / 2. + j as f32 * PI / 7.;
            outline.push(b + d * (t.cos() * 3.5) + n * (t.sin() * 3.5));
        }
        for j in 0..8 {
            let t = PI / 2. + j as f32 * PI / 7.;
            outline.push(a + d * (t.cos() * 5.) + n * (t.sin() * 5.));
        }
        prism(
            &mut m,
            &outline,
            0.7,
            5.,
            if (i == 0 && g.left) || (i == 1 && g.right) {
                10
            } else {
                8
            },
        );
        cylinder(&mut m, a, 2.2, 6., 0.5, 5);
    }
    if effects {
        for (i, t) in g.trail.iter().enumerate().step_by(2) {
            ring(
                &mut m,
                *t,
                1.4 + i as f32 * 0.08,
                0.6,
                0.5,
                if i > 7 { 7 } else { 5 },
            );
        }
    }
    // A faceted sphere with quantised highlights. Height and perspective are real.
    let lift = if g.rail_active {
        g.active_rail().height(g.rail_distance)
    } else {
        0.
    };
    let center = P3::new(g.ball.x, g.ball.y, 4.7 + lift);
    let r = 4.;
    for row in 0..5 {
        let latitude = PI * row as f32 / 5.;
        let next = PI * (row + 1) as f32 / 5.;
        for col in 0..10 {
            let a = TAU * col as f32 / 10.;
            let b = TAU * (col + 1) as f32 / 10.;
            let p = |t: f32, l: f32| {
                P3::new(
                    center.x + r * l.sin() * t.cos(),
                    center.y + r * l.sin() * t.sin(),
                    center.z + r * l.cos(),
                )
            };
            let pts = vec![p(a, latitude), p(b, latitude), p(b, next), p(a, next)];
            let light = (a.cos() * 0.3 - a.sin() * 0.4 + latitude.cos() * 0.7).max(-0.5);
            let ink = if light > 0.6 {
                8
            } else if light > 0.2 {
                7
            } else if light > -0.2 {
                6
            } else {
                5
            };
            face(&mut m, pts, ink, false);
        }
    }
    // Live circuit inserts and charge segments.
    for i in 0..3 {
        cylinder(
            &mut m,
            V::new(95. + i as f32 * 16., 273.),
            3.,
            0.,
            0.5,
            if g.lamps[i] { 10 } else { 5 },
        );
    }
    for i in 0..8 {
        rect(
            &mut m,
            208.,
            343. - i as f32 * 4.,
            8.,
            2.,
            0.4,
            if g.charge > i as f32 / 8. { 10 } else { 4 },
        );
    }
    m
}
pub fn project_sorted(camera: &Camera, faces: &[Face]) -> Vec<Projected> {
    let mut result = faces.iter().map(|f| camera.face(f)).collect::<Vec<_>>();
    result.sort_by(|a, b| {
        b.floor.cmp(&a.floor).then_with(|| {
            if a.floor && b.floor {
                std::cmp::Ordering::Equal
            } else {
                b.depth.total_cmp(&a.depth)
            }
        })
    });
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn perspective_has_foreshortening_and_height() {
        let c = Camera::new(700., 800.);
        let top =
            (c.project(P3::new(244., 0., 0.)).0[0] - c.project(P3::new(0., 0., 0.)).0[0]).abs();
        let bottom =
            (c.project(P3::new(244., 380., 0.)).0[0] - c.project(P3::new(0., 380., 0.)).0[0]).abs();
        assert!(bottom > top * 1.2);
        assert!(c.project(P3::new(122., 190., 20.)).0[1] < c.project(P3::new(122., 190., 0.)).0[1]);
    }
    #[test]
    fn projection_stays_finite_and_in_view() {
        let c = Camera::new(600., 720.);
        for board in crate::board::ALL {
            for face in static_table(board) {
                for p in face.points {
                    let (q, _) = c.project(p);
                    assert!(q[0].is_finite() && q[1].is_finite());
                    assert!(
                        q[0] > -10. && q[0] < 610. && q[1] > -10. && q[1] < 730.,
                        "{}: {:?}",
                        board.key,
                        q
                    );
                }
            }
        }
    }
}
