//! Authored cabinet toys and surface detail. Raised toys stay behind the top
//! boundary or outside the side rails; playfield decoration is flush artwork.
//! These meshes are built once and share the live scene's palette materials.
use super::{Face, P3, face, lettering, plate, prism, rect, ring, strip};
use crate::{
    board::Board,
    physics::{Game, V},
};
use std::f32::consts::TAU;

fn block(m: &mut Vec<Face>, c: V, w: f32, d: f32, z: f32, h: f32, ink: usize) {
    prism(
        m,
        &[
            c + V::new(-w / 2., -d / 2.),
            c + V::new(w / 2., -d / 2.),
            c + V::new(w / 2., d / 2.),
            c + V::new(-w / 2., d / 2.),
        ],
        z,
        h,
        ink,
    );
}
fn column(m: &mut Vec<Face>, c: V, r: f32, z: f32, h: f32, ink: usize) {
    let outline = (0..8)
        .map(|i| {
            let a = i as f32 * TAU / 8.;
            c + V::new(a.cos(), a.sin()) * r
        })
        .collect::<Vec<_>>();
    prism(m, &outline, z, h, ink);
}
fn taper(m: &mut Vec<Face>, c: V, r: f32, top: f32, z: f32, h: f32, ink: usize) {
    let p = |i: usize, r: f32, z: f32| {
        let a = i as f32 * TAU / 8.;
        P3::new(c.x + a.cos() * r, c.y + a.sin() * r, z)
    };
    for i in 0..8 {
        face(
            m,
            vec![
                p(i, r, z),
                p(i + 1, r, z),
                p(i + 1, top, z + h),
                p(i, top, z + h),
            ],
            if i < 4 { ink } else { 5 },
            false,
        );
    }
    face(
        m,
        (0..8).map(|i| p(i, top, z + h)).collect::<Vec<_>>(),
        7,
        false,
    );
}
fn beam(m: &mut Vec<Face>, a: P3, b: P3, r: f32, ink: usize) {
    let dir = b.sub(a).unit();
    let u = dir
        .cross(if dir.z.abs() > 0.9 {
            P3::new(0., 1., 0.)
        } else {
            P3::new(0., 0., 1.)
        })
        .unit();
    let v = dir.cross(u);
    let p = |c: P3, s: f32, t: f32| {
        P3::new(
            c.x + (u.x * s + v.x * t) * r,
            c.y + (u.y * s + v.y * t) * r,
            c.z + (u.z * s + v.z * t) * r,
        )
    };
    let edges = [(-1., -1.), (1., -1.), (1., 1.), (-1., 1.)];
    for i in 0..4 {
        let (s, t) = edges[i];
        let (s1, t1) = edges[(i + 1) % 4];
        face(
            m,
            vec![p(a, s, t), p(b, s, t), p(b, s1, t1), p(a, s1, t1)],
            if i == 1 {
                ink
            } else if i == 2 {
                6
            } else {
                4
            },
            false,
        );
    }
    for c in [a, b] {
        face(
            m,
            edges.iter().map(|&(s, t)| p(c, s, t)).collect::<Vec<_>>(),
            ink,
            false,
        );
    }
}
fn wheel(m: &mut Vec<Face>, c: P3, r: f32, spokes: usize) {
    // A vertical wheel: twelve faceted rim sections, open center, raised spokes.
    for i in 0..12 {
        let a = i as f32 * TAU / 12.;
        let b = (i + 1) as f32 * TAU / 12.;
        let p = |a: f32, r: f32, y: f32| P3::new(c.x + r * a.cos(), y, c.z + r * a.sin());
        face(
            m,
            vec![
                p(a, r, c.y + 1.),
                p(b, r, c.y + 1.),
                p(b, r - 2., c.y + 1.),
                p(a, r - 2., c.y + 1.),
            ],
            7,
            false,
        );
        face(
            m,
            vec![
                p(a, r, c.y - 1.),
                p(b, r, c.y - 1.),
                p(b, r, c.y + 1.),
                p(a, r, c.y + 1.),
            ],
            5,
            false,
        );
    }
    for i in 0..spokes {
        let a = i as f32 * TAU / spokes as f32;
        beam(
            m,
            c,
            P3::new(c.x + (r - 2.) * a.cos(), c.y, c.z + (r - 2.) * a.sin()),
            0.7,
            6,
        );
    }
    block(m, V::new(c.x, c.y + 1.), 4., 3., c.z - 2., 4., 8);
}
fn front_sign(m: &mut Vec<Face>, c: V, z: f32, w: f32, text: &str) {
    block(m, c, w, 2., z, 8., 3);
    let mut label = Vec::new();
    lettering(&mut label, 0., 0., 0.8, text, 7);
    for mut f in label {
        for p in &mut f.points {
            let row = p.y;
            p.x += c.x - text.len() as f32 * 1.6;
            p.y = c.y + 1.15;
            p.z = z + 6. - row;
        }
        f.floor = false;
        m.push(f);
    }
}
fn grille(m: &mut Vec<Face>, c: V, z: f32, w: f32, rows: usize) {
    for i in 0..rows {
        let h = z + i as f32 * 2.4;
        face(
            m,
            vec![
                P3::new(c.x - w / 2., c.y, h),
                P3::new(c.x + w / 2., c.y, h),
                P3::new(c.x + w / 2., c.y, h + 0.9),
                P3::new(c.x - w / 2., c.y, h + 0.9),
            ],
            0,
            false,
        );
    }
}
fn raised_ring(m: &mut Vec<Face>, c: V, r: f32, w: f32, z: f32, ink: usize) {
    let begin = m.len();
    ring(m, c, r, w, z, ink);
    for f in &mut m[begin..] {
        f.floor = false;
    }
}

fn ferrous(m: &mut Vec<Face>) {
    for x in [65., 179.] {
        block(m, V::new(x, 7.), 12., 16., 5., 4., 5);
        block(m, V::new(x, 7.), 6., 9., 9., 36., 6);
        block(m, V::new(x, 7.), 11., 14., 43., 5., 7);
        beam(
            m,
            P3::new(x, 12., 12.),
            P3::new(x + if x < 120. { 18. } else { -18. }, 12., 43.),
            1.1,
            5,
        );
    }
    block(m, V::new(122., 7.), 125., 11., 46., 7., 6);
    for x in (72..179).step_by(12) {
        block(m, V::new(x as f32, 13.), 2., 1., 48., 2., 8);
    }
    wheel(m, P3::new(122., 10., 29.), 17., 8);
    beam(m, P3::new(93., 6., 11.), P3::new(102., 6., 29.), 2., 7);
    beam(m, P3::new(151., 6., 11.), P3::new(142., 6., 29.), 2., 7);
    front_sign(m, V::new(122., 15.), 5., 57., "FE WORKS");
}
fn monolith(m: &mut Vec<Face>) {
    for (x, h, r) in [(79., 29., 7.), (122., 49., 10.), (165., 34., 7.)] {
        let c = V::new(x, 7.);
        prism(
            m,
            &[
                c + V::new(-r, 0.),
                c + V::new(0., -7.),
                c + V::new(r, 0.),
                c + V::new(0., 7.),
            ],
            5.,
            h,
            7,
        );
        for row in 0..4 {
            let z = 12. + row as f32 * 6.;
            beam(m, P3::new(x - 3., 12., z), P3::new(x + 3., 12., z), 0.45, 8);
        }
    }
    for side in [-1., 1.] {
        beam(
            m,
            P3::new(122. + side * 26., 8., 8.),
            P3::new(122. + side * 20., 8., 35.),
            0.6,
            6,
        );
        beam(
            m,
            P3::new(122. + side * 20., 8., 35.),
            P3::new(122. + side * 12., 8., 43.),
            0.6,
            7,
        );
    }
    front_sign(m, V::new(122., 16.), 5., 67., "NULL SIGNAL");
}
fn foundry(m: &mut Vec<Face>) {
    for x in [80., 164.] {
        column(m, V::new(x, 7.), 7., 5., 9., 5);
        column(m, V::new(x, 7.), 4., 14., 27., 8);
        column(m, V::new(x, 7.), 7., 39., 7., 6);
        for z in [17., 23., 29., 35.] {
            raised_ring(m, V::new(x, 7.), 4.4, 0.8, z, 6);
        }
    }
    block(m, V::new(122., 7.), 102., 15., 44., 8., 6);
    block(m, V::new(122., 8.), 46., 12., 29., 9., 7);
    grille(m, V::new(122., 14.2), 31., 34., 3);
    block(m, V::new(122., 8.), 53., 14., 8., 5., 7);
    wheel(m, P3::new(122., 11., 21.), 9., 6);
    for x in [99., 109., 135., 145.] {
        block(m, V::new(x, 15.), 4., 3., 5., 2., 5);
    }
    front_sign(m, V::new(122., 16.), 5., 74., "HOT METAL");
}
fn breakwater(m: &mut Vec<Face>) {
    let c = V::new(150., 7.);
    column(m, c, 12., 5., 4., 5);
    taper(m, c, 9., 5.5, 9., 30., 7);
    raised_ring(m, c, 9., 1.4, 40., 7);
    column(m, c, 7., 39., 8., 3);
    for dx in [-5., 5.] {
        for dy in [-4., 4.] {
            beam(
                m,
                P3::new(c.x + dx, c.y + dy, 39.),
                P3::new(c.x + dx, c.y + dy, 48.),
                0.55,
                8,
            );
        }
    }
    taper(m, c, 10., 1., 48., 7., 6);
    block(m, V::new(79., 7.), 27., 13., 5., 8., 5);
    prism(
        m,
        &[V::new(63., 1.), V::new(95., 1.), V::new(79., 7.)],
        13.,
        8.,
        7,
    );
    for i in 0..3 {
        let x = 98. + i as f32 * 9.;
        beam(m, P3::new(x, 11., 8.), P3::new(x + 4., 11., 14.), 0.8, 7);
        beam(
            m,
            P3::new(x + 4., 11., 14.),
            P3::new(x + 9., 11., 8.),
            0.8,
            5,
        );
    }
    front_sign(m, V::new(122., 16.), 5., 69., "NORTH LIGHT");
}
fn switchyard(m: &mut Vec<Face>) {
    block(m, V::new(73., 7.), 20., 14., 5., 31., 6);
    block(m, V::new(73., 7.), 27., 17., 35., 3., 7);
    grille(m, V::new(73., 14.2), 23., 13., 3);
    beam(m, P3::new(73., 6., 38.), P3::new(73., 6., 49.), 1., 7);
    block(m, V::new(73., 7.), 9., 4., 44., 6., 3);
    let c = V::new(143., 7.);
    prism(
        m,
        &[
            c + V::new(-34., -5.),
            c + V::new(29., -5.),
            c + V::new(36., 0.),
            c + V::new(29., 6.),
            c + V::new(-34., 6.),
        ],
        10.,
        7.,
        6,
    );
    block(m, V::new(119., 7.), 16., 13., 17., 14., 7);
    block(m, V::new(119., 7.), 20., 16., 31., 2., 5);
    block(m, V::new(150., 7.), 36., 9., 17., 7., 7);
    column(m, V::new(155., 7.), 3., 24., 10., 5);
    for x in [123., 143., 163.] {
        wheel(m, P3::new(x, 13.5, 10.), 4., 4);
    }
    for x in (132..174).step_by(7) {
        block(m, V::new(x as f32, 12.), 1., 1., 19., 4., 5);
    }
    front_sign(m, V::new(122., 16.), 5., 69., "TRACK NINE");
}
fn drydock(m: &mut Vec<Face>) {
    for x in [57., 69.] {
        beam(m, P3::new(x, 7., 5.), P3::new(63., 7., 50.), 1.2, 7);
    }
    beam(m, P3::new(63., 7., 50.), P3::new(170., 7., 50.), 1.5, 7);
    beam(m, P3::new(65., 7., 35.), P3::new(140., 7., 50.), 0.9, 5);
    beam(m, P3::new(159., 7., 49.), P3::new(159., 7., 27.), 0.5, 6);
    wheel(m, P3::new(159., 9., 24.), 3.5, 0);
    prism(
        m,
        &[
            V::new(90., 0.),
            V::new(142., 0.),
            V::new(157., 7.),
            V::new(142., 14.),
            V::new(90., 14.),
            V::new(82., 7.),
        ],
        5.,
        7.,
        6,
    );
    block(m, V::new(110., 7.), 18., 10., 12., 8., 7);
    block(m, V::new(129., 7.), 13., 9., 12., 5., 5);
    beam(m, P3::new(136., 7., 16.), P3::new(136., 7., 34.), 0.65, 7);
    beam(m, P3::new(129., 7., 29.), P3::new(143., 7., 29.), 0.5, 7);
    for x in [182., 194.] {
        block(m, V::new(x, 7.), 10., 12., 5., 10., 5);
        for z in [8., 12.] {
            grille(m, V::new(x, 13.1), z, 7., 1);
        }
    }
    front_sign(m, V::new(122., 16.), 5., 66., "PORT OF FE");
}
fn blueprint(m: &mut Vec<Face>) {
    // A wireframe globe above a drafting arm, with no opaque sphere in the way.
    let c = P3::new(137., 6., 31.);
    for plane in 0..3 {
        for i in 0..12 {
            let p = |i: usize| {
                let a = i as f32 * TAU / 12.;
                let s = 17. * a.sin();
                let t = 17. * a.cos();
                match plane {
                    0 => P3::new(c.x + s, c.y, c.z + t),
                    1 => P3::new(c.x, c.y + s * 0.4, c.z + t),
                    _ => P3::new(c.x + s, c.y + t * 0.4, c.z),
                }
            };
            beam(m, p(i), p(i + 1), 0.45, 7);
        }
    }
    column(m, V::new(c.x, c.y), 6., 5., 7., 5);
    beam(m, P3::new(c.x, c.y, 12.), P3::new(c.x, c.y, 47.), 0.65, 6);
    let joints = [
        P3::new(67., 7., 10.),
        P3::new(83., 7., 39.),
        P3::new(108., 7., 24.),
    ];
    for pair in joints.windows(2) {
        beam(m, pair[0], pair[1], 1.7, 7);
    }
    for p in joints {
        wheel(m, P3::new(p.x, p.y + 1., p.z), 3.5, 0);
    }
    for i in 0..9 {
        block(
            m,
            V::new(91. + i as f32 * 8., 13.),
            1.,
            1.,
            6.,
            if i % 2 == 0 { 4. } else { 2. },
            7,
        );
    }
    front_sign(m, V::new(122., 16.), 5., 73., "DRAFT 03 A");
}
fn critical(m: &mut Vec<Face>) {
    let c = V::new(122., 7.);
    column(m, c, 14., 5., 7., 5);
    taper(m, c, 11., 7., 12., 29., 6);
    for z in [16., 22., 28., 34.] {
        raised_ring(m, c, 12., 2.2, z, 7);
    }
    for x in [-16., 16.] {
        for y in [-5., 5.] {
            beam(
                m,
                P3::new(c.x + x, c.y + y, 7.),
                P3::new(c.x + x, c.y + y, 48.),
                0.8,
                7,
            );
        }
    }
    block(m, c, 36., 14., 46., 3., 6);
    for x in [73., 171.] {
        column(m, V::new(x, 7.), 8., 5., 23., 6);
        for z in [9., 14., 19., 24.] {
            raised_ring(m, V::new(x, 7.), 9., 1.5, z, 7);
        }
        block(m, V::new(x, 7.), 12., 9., 29., 3., 7);
        beam(
            m,
            P3::new(x, 7., 31.),
            P3::new(if x < 120. { 103. } else { 141. }, 7., 43.),
            0.7,
            6,
        );
    }
    front_sign(m, V::new(122., 16.), 5., 77., "CONTAINMENT");
}
fn oxide(m: &mut Vec<Face>) {
    wheel(m, P3::new(122., 10., 29.), 18., 5);
    for side in [-1., 1.] {
        let points = [
            P3::new(122. + side * 48., 7., 8.),
            P3::new(122. + side * 48., 7., 30.),
            P3::new(122. + side * 33., 7., 39.),
            P3::new(122. + side * 21., 7., 32.),
        ];
        for p in points.windows(2) {
            beam(m, p[0], p[1], 2., 6);
        }
        column(m, V::new(122. + side * 48., 7.), 5., 6., 4., 7);
        for i in 0..3 {
            let x = 122. + side * (57. + i as f32 * 5.);
            let z = 15. + i as f32 * 8.;
            beam(
                m,
                P3::new(x, 8., z - 5.),
                P3::new(x - side * 8., 8., z + 4.),
                0.5,
                5,
            );
            face(
                m,
                vec![
                    P3::new(x, 8., z),
                    P3::new(x + side * 7., 8., z + 3.),
                    P3::new(x + side * 3., 8., z + 8.),
                ],
                6,
                false,
            );
        }
    }
    front_sign(m, V::new(122., 16.), 5., 76., "RECLAIM FLOW");
}
fn blackout(m: &mut Vec<Face>) {
    for x in [66., 178.] {
        block(m, V::new(x, 7.), 23., 14., 5., 34., 3);
        block(m, V::new(x, 7.), 25., 16., 39., 2., 6);
        for z in [15., 29.] {
            wheel(m, P3::new(x, 14.5, z), 7., 0);
        }
        for row in 0..3 {
            block(
                m,
                V::new(x - 7. + row as f32 * 7., 15.),
                1.2,
                1.,
                7.,
                1.2,
                7,
            );
        }
    }
    let bolt = [
        P3::new(128., 7., 55.),
        P3::new(108., 7., 32.),
        P3::new(128., 7., 35.),
        P3::new(112., 7., 10.),
    ];
    for pair in bolt.windows(2) {
        beam(m, pair[0], pair[1], 2.5, 8);
    }
    for x in [91., 153.] {
        beam(m, P3::new(x, 6., 6.), P3::new(x, 6., 22.), 0.7, 5);
    }
    front_sign(m, V::new(122., 16.), 5., 78., "HIGH VOLTAGE");
}

fn badge(m: &mut Vec<Face>, c: V, design: u8) {
    // Small raised rim hardware stays within the existing bumper envelope.
    for i in 0..4 {
        let a = i as f32 * TAU / 4.;
        block(
            m,
            c + V::new(a.cos(), a.sin()) * 12.,
            1.4,
            1.4,
            20.3,
            0.8,
            7,
        );
    }
    let begin = m.len();
    for i in 0..8 {
        let a = i as f32 * TAU / 8.;
        let d = V::new(a.cos(), a.sin());
        let t = V::new(-d.y, d.x);
        match design {
            0 | 2 => strip(m, c + d * 9., c + d * 11.4, 0.45, 21.2, 0., 6),
            1 | 4 => strip(m, c + d * 9.5 - t, c + d * 9.5 + t, 0.35, 21.2, 0., 6),
            3 | 5 => plate(m, &[c + d * 9., c + d * 11. + t, c + d * 11. - t], 21.2, 6),
            6 => strip(m, c + d * 8.8, c + d * 11.4 + t, 0.3, 21.2, 0., 7),
            7 | 8 => strip(m, c + d * 9., c + d * 10. + t * 1.5, 0.5, 21.2, 0., 6),
            _ => plate(m, &[c + d * 9., c + d * 11. + t, c + d * 10. - t], 21.2, 7),
        }
    }
    for f in &mut m[begin..] {
        f.floor = false;
    }
}

fn field_emblem(m: &mut Vec<Face>, design: u8) {
    let c = V::new(113., 309.);
    let line = |m: &mut Vec<Face>, a: V, b: V| strip(m, c + a, c + b, 0.6, 0.4, 0., 5);
    match design {
        0 => {
            ring(m, c, 13., 1.3, 0.4, 5);
            ring(m, c, 7., 0.8, 0.4, 6);
            for i in 0..8 {
                let a = i as f32 * TAU / 8.;
                let d = V::new(a.cos(), a.sin());
                strip(m, c + d * 11., c + d * 17., 1.2, 0.4, 0., 5);
            }
        }
        1 => {
            for r in [8., 14.] {
                let p = [
                    V::new(0., -r),
                    V::new(r * 0.7, 0.),
                    V::new(0., r),
                    V::new(-r * 0.7, 0.),
                ];
                for i in 0..4 {
                    line(m, p[i], p[(i + 1) % 4]);
                }
            }
            line(m, V::new(-20., 0.), V::new(-12., 0.));
            line(m, V::new(12., 0.), V::new(20., 0.));
        }
        2 => {
            for x in [-12., 12.] {
                line(m, V::new(x, -14.), V::new(x, 13.));
            }
            rect(m, c.x - 15., c.y - 14., 30., 3., 0.4, 5);
            rect(m, c.x - 8., c.y - 6., 16., 6., 0.4, 4);
            rect(m, c.x - 14., c.y + 11., 28., 3., 0.4, 5);
            for x in [-5., 0., 5.] {
                line(m, V::new(x, 2.), V::new(x + 2., 7.));
            }
        }
        3 => {
            line(m, V::new(-5., 8.), V::new(-2., -9.));
            line(m, V::new(5., 8.), V::new(2., -9.));
            rect(m, c.x - 5., c.y - 12., 10., 3., 0.4, 5);
            line(m, V::new(0., -16.), V::new(-6., -12.));
            line(m, V::new(0., -16.), V::new(6., -12.));
            for side in [-1., 1.] {
                for i in 0..3 {
                    line(
                        m,
                        V::new(side * 8., -10. + i as f32 * 4.),
                        V::new(side * 18., -15. + i as f32 * 9.),
                    );
                }
            }
            line(m, V::new(-14., 12.), V::new(0., 15.));
            line(m, V::new(0., 15.), V::new(14., 12.));
        }
        4 => {
            for side in [-1., 1.] {
                line(m, V::new(side * 5., 14.), V::new(side * 5., -12.));
                line(m, V::new(side * 5., 3.), V::new(side * 15., -13.));
            }
            for y in [-9., -3., 3., 9.] {
                line(m, V::new(-9., y), V::new(9., y));
            }
        }
        5 => {
            ring(m, c + V::new(0., -10.), 3., 0.8, 0.4, 6);
            line(m, V::new(0., -7.), V::new(0., 13.));
            line(m, V::new(-8., -2.), V::new(8., -2.));
            for side in [-1., 1.] {
                line(m, V::new(0., 13.), V::new(side * 12., 6.));
                line(m, V::new(side * 12., 6.), V::new(side * 14., 0.));
                line(m, V::new(side * 14., 0.), V::new(side * 8., 4.));
            }
        }
        6 => {
            ring(m, c, 12., 0.5, 0.4, 5);
            for side in [-1., 1.] {
                line(m, V::new(0., -15.), V::new(side * 10., 14.));
                line(m, V::new(side * 10., 14.), V::new(side * 5., 11.));
            }
            ring(m, c + V::new(0., -6.), 2., 0.6, 0.4, 6);
        }
        7 => {
            ring(m, c, 13., 1., 0.4, 5);
            ring(m, c, 4., 1., 0.4, 6);
            for i in 0..3 {
                let a = i as f32 * TAU / 3.;
                let d = V::new(a.cos(), a.sin());
                let t = V::new(-d.y, d.x);
                plate(
                    m,
                    &[
                        c + d * 6. - t * 2.,
                        c + d * 11. - t * 5.,
                        c + d * 11. + t * 5.,
                        c + d * 6. + t * 2.,
                    ],
                    0.4,
                    4,
                );
            }
        }
        8 => {
            line(m, V::new(-4., 14.), V::new(2., -15.));
            for i in 0..4 {
                let y = -9. + i as f32 * 6.;
                let side = if i % 2 == 0 { 1. } else { -1. };
                line(m, V::new(-1., y + 5.), V::new(side * 9., y - 2.));
                plate(
                    m,
                    &[
                        c + V::new(side * 5., y),
                        c + V::new(side * 13., y - 7.),
                        c + V::new(side * 12., y + 1.),
                    ],
                    0.4,
                    5,
                );
            }
        }
        _ => {
            plate(
                m,
                &[
                    c + V::new(5., -17.),
                    c + V::new(-9., 2.),
                    c + V::new(2., 0.),
                    c + V::new(-5., 17.),
                    c + V::new(11., -5.),
                    c + V::new(0., -3.),
                ],
                0.4,
                5,
            );
            for side in [-1., 1.] {
                line(m, V::new(side * 16., -9.), V::new(side * 16., 8.));
            }
        }
    }
}

fn cabinet_detail(m: &mut Vec<Face>, board: &Board) {
    // Recessed service plates and flush warning hatch keep the live field clear.
    for (side, x) in [(0, 9.), (1, 235.)] {
        for row in 0..3 {
            let y = 91. + row as f32 * 83.;
            block(m, V::new(x, y), 8., 35., 10., 0.8, 3);
            for dy in [-13., 13.] {
                block(m, V::new(x, y + dy), 1.4, 1.4, 10.9, 0.5, 7);
            }
            for i in 0..5 {
                let dy = -8. + i as f32 * 4.;
                rect(m, x - 2.5, y + dy, 5., 0.65, 11., 5);
            }
            // Convert just the vent panels' horizontal surface marks to solids.
            for f in m.iter_mut().rev().take(5) {
                f.floor = false;
            }
            for i in 0..3 {
                let y1 = y + 20. + i as f32 * 3.;
                rect(
                    m,
                    x - 2.,
                    y1,
                    4.,
                    1.,
                    10.4,
                    if (i + board.design as usize + side).is_multiple_of(2) {
                        6
                    } else {
                        4
                    },
                );
                m.last_mut().unwrap().floor = false;
            }
        }
    }
    // An individually stenciled apron and printed emblem beneath the flippers.
    let short = match board.design {
        0 => "FORGED IN FE",
        1 => "ZERO MATTER",
        2 => "PRESSURE 09",
        3 => "KEEP THE LIGHT",
        4 => "NIGHT FREIGHT",
        5 => "DEEP WATER",
        6 => "DRAW THE LINE",
        7 => "CORE ONLINE",
        8 => "PATINA WORKS",
        _ => "AFTER HOURS",
    };
    lettering(m, 132., 354., 0.95, short, 6);
    for i in 0..6 {
        let x = 51. + i as f32 * 5.;
        strip(m, V::new(x, 350.), V::new(x + 3., 353.), 0.6, 0.4, 0., 5);
    }
    let c = V::new(113., 337.);
    match board.design {
        0 | 2 => {
            ring(m, c, 9., 0.7, 0.4, 5);
            for i in 0..8 {
                let a = i as f32 * TAU / 8.;
                let d = V::new(a.cos(), a.sin());
                strip(m, c + d * 7., c + d * 11., 0.7, 0.4, 0., 5);
            }
        }
        1 => plate(
            m,
            &[
                c + V::new(0., -11.),
                c + V::new(8., 0.),
                c + V::new(0., 10.),
                c + V::new(-8., 0.),
            ],
            0.4,
            4,
        ),
        3 | 5 => {
            for i in 0..3 {
                let y = c.y - 7. + i as f32 * 5.;
                strip(m, V::new(103., y), V::new(113., y + 3.), 0.5, 0.4, 0., 5);
                strip(m, V::new(113., y + 3.), V::new(123., y), 0.5, 0.4, 0., 5);
            }
        }
        4 => {
            for x in [108., 118.] {
                strip(m, V::new(x, 327.), V::new(x, 347.), 0.6, 0.4, 0., 5);
            }
            for y in (329..346).step_by(4) {
                rect(m, 105., y as f32, 16., 0.7, 0.4, 4);
            }
        }
        6 => {
            ring(m, c, 10., 0.5, 0.4, 5);
            strip(
                m,
                c + V::new(-13., 0.),
                c + V::new(13., 0.),
                0.4,
                0.4,
                0.,
                5,
            );
            strip(
                m,
                c + V::new(0., -13.),
                c + V::new(0., 13.),
                0.4,
                0.4,
                0.,
                5,
            );
        }
        7 => {
            ring(m, c, 3., 1., 0.4, 5);
            for i in 0..3 {
                let a = i as f32 * TAU / 3.;
                let d = V::new(a.cos(), a.sin());
                let t = V::new(-d.y, d.x);
                plate(
                    m,
                    &[
                        c + d * 5. - t * 2.,
                        c + d * 10. - t * 5.,
                        c + d * 10. + t * 5.,
                        c + d * 5. + t * 2.,
                    ],
                    0.4,
                    4,
                );
            }
        }
        8 => {
            for side in [-1., 1.] {
                for i in 0..3 {
                    strip(
                        m,
                        c + V::new(0., -8. + i as f32 * 5.),
                        c + V::new(side * 8., -5. + i as f32 * 5.),
                        0.6,
                        0.4,
                        0.,
                        5,
                    );
                }
            }
        }
        _ => plate(
            m,
            &[
                c + V::new(3., -12.),
                c + V::new(-7., 1.),
                c + V::new(2., -1.),
                c + V::new(-3., 11.),
                c + V::new(8., -4.),
                c + V::new(0., -2.),
            ],
            0.4,
            5,
        ),
    }
    // Shot arrows are flush inlays adjacent to the actual rail mouths.
    for (entry, label) in [
        (board.orbit.points[0], "ORBIT"),
        (board.ramp.points[0], "RAMP"),
    ] {
        let x = entry.x;
        if !(55. ..170.).contains(&x) {
            let sign = if x < 120. { 1. } else { -1. };
            let c = entry + V::new(sign * 13., 12.);
            plate(
                m,
                &[c + V::new(-3., 5.), c + V::new(0., 0.), c + V::new(3., 5.)],
                0.4,
                6,
            );
            lettering(m, c.x - 8., c.y + 8., 0.75, label, 5);
        }
    }
}

pub(super) fn decorate(m: &mut Vec<Face>, board: &Board) {
    cabinet_detail(m, board);
    field_emblem(m, board.design);
    for &c in board.bumpers {
        badge(m, c, board.design);
    }
    block(m, V::new(122., 7.), 167., 16., 0.8, 4., 3);
    match board.design {
        0 => ferrous(m),
        1 => monolith(m),
        2 => foundry(m),
        3 => breakwater(m),
        4 => switchyard(m),
        5 => drydock(m),
        6 => blueprint(m),
        7 => critical(m),
        8 => oxide(m),
        _ => blackout(m),
    }
    // Mechanical score lamps on a front-facing strip, lit only by progression.
    for x in [43., 48., 53., 191., 196., 201.] {
        block(m, V::new(x, 16.), 3.4, 1.4, 4.8, 3.4, 4);
    }
}
pub(super) fn progress_lights(m: &mut Vec<Face>, g: &Game) {
    for i in 0..3 {
        for (x, on) in [
            (43. + i as f32 * 5., g.lamps[i]),
            (
                191. + i as f32 * 5.,
                g.overdrive > 0.
                    || match i {
                        0 => g.target_progress > 0,
                        1 => g.ramp_open,
                        _ => g.jackpot_ready,
                    },
            ),
        ] {
            face(
                m,
                vec![
                    P3::new(x - 1., 16.8, 5.5),
                    P3::new(x + 1., 16.8, 5.5),
                    P3::new(x + 1., 16.8, 7.5),
                    P3::new(x - 1., 16.8, 7.5),
                ],
                if on { 10 } else { 5 },
                false,
            );
        }
    }
}
