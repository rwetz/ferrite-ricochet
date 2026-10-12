use crate::{
    physics::Game,
    profile,
    scene::{self, Camera, Projected},
};
use ferrite_design::prelude::*;
use gpui::{Bounds, IntoElement, Path, PathBuilder, Pixels, Window, canvas, point, px};
use std::{cell::RefCell, time::Instant};
struct Draw {
    path: Path<Pixels>,
    ink: usize,
    depth: f32,
}
struct Cached {
    board_key: &'static str,
    key: [f32; 4],
    camera: Camera,
    floor: Vec<Draw>,
    solid: Vec<Draw>,
}
thread_local! {static CACHE:RefCell<Option<Cached>>=const{RefCell::new(None)};}
fn path(f: &Projected, origin: [f32; 2]) -> Path<Pixels> {
    let mut b = PathBuilder::fill();
    for (i, p) in f.points.iter().enumerate() {
        let v = point(px(origin[0] + p[0]), px(origin[1] + p[1]));
        if i == 0 {
            b.move_to(v);
        } else {
            b.line_to(v);
        }
    }
    b.close();
    b.build().expect("convex mesh face")
}
fn floor_batches(faces: &[Projected], origin: [f32; 2]) -> Vec<Draw> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < faces.len() {
        if !faces[i].floor {
            i += 1;
            continue;
        }
        let ink = faces[i].ink;
        let mut b = PathBuilder::fill();
        while i < faces.len() && faces[i].floor && faces[i].ink == ink {
            for (j, p) in faces[i].points.iter().enumerate() {
                let v = point(px(origin[0] + p[0]), px(origin[1] + p[1]));
                if j == 0 {
                    b.move_to(v);
                } else {
                    b.line_to(v);
                }
            }
            b.close();
            i += 1;
        }
        out.push(Draw {
            path: b.build().expect("floor mesh"),
            ink,
            depth: 0.,
        });
    }
    out
}
fn draw(face: &Draw, offset: [f32; 2], colors: &[gpui::Hsla; 13], window: &mut Window) {
    let mut path = face.path.clone();
    let delta = point(px(offset[0]), px(offset[1]));
    path.bounds.origin += delta;
    for vertex in &mut path.vertices {
        vertex.xy_position += delta;
    }
    window.paint_path(path, colors[face.ink]);
}
pub fn table(g: Game, effects: bool, cx: &gpui::App) -> impl IntoElement {
    let p = palette(cx);
    let colors = [
        hsla(p.sunken),
        hsla(p.bg),
        hsla(p.surface),
        hsla(p.raised),
        hsla(p.line),
        hsla(p.line_strong),
        hsla(p.fg_faint),
        hsla(p.fg_dim),
        hsla(p.fg),
        hsla(p.accent_dim),
        hsla(p.accent),
        hsla(p.warning),
        hsla(p.danger),
    ];
    canvas(
        |_, _, _| {},
        move |bounds: Bounds<Pixels>, _, window: &mut Window, cx| {
            let begin = Instant::now();
            let key = [
                f32::from(bounds.left()),
                f32::from(bounds.top()),
                f32::from(bounds.size.width),
                f32::from(bounds.size.height),
            ];
            let origin = [key[0], key[1]];
            CACHE.with_borrow_mut(|cache| {
                if cache
                    .as_ref()
                    .is_none_or(|c| c.key != key || c.board_key != g.board.key)
                {
                    let camera = Camera::new(key[2], key[3]);
                    let faces = scene::project_sorted(&camera, &scene::static_table(g.board));
                    let floor = floor_batches(&faces, origin);
                    let solid = faces
                        .iter()
                        .filter(|f| !f.floor)
                        .map(|f| Draw {
                            path: path(f, origin),
                            ink: f.ink,
                            depth: f.depth,
                        })
                        .collect();
                    *cache = Some(Cached {
                        board_key: g.board.key,
                        key,
                        camera,
                        floor,
                        solid,
                    });
                }
                let cached = cache.as_ref().unwrap();
                let offset = if effects {
                    [
                        (g.time * 80.).sin().signum() * g.shaking * 4.,
                        (g.time * 57.).cos().signum() * g.shaking * 2.,
                    ]
                } else {
                    [0.; 2]
                };
                for face in &cached.floor {
                    draw(face, offset, &colors, window);
                }
                let dynamic =
                    scene::project_sorted(&cached.camera, &scene::dynamic_table(&g, effects));
                let floor = floor_batches(&dynamic, origin);
                for face in &floor {
                    draw(face, offset, &colors, window);
                }
                let solid = dynamic
                    .iter()
                    .filter(|f| !f.floor)
                    .map(|f| Draw {
                        path: path(f, origin),
                        ink: f.ink,
                        depth: f.depth,
                    })
                    .collect::<Vec<_>>();
                let mut a = 0;
                let mut b = 0;
                while a < cached.solid.len() || b < solid.len() {
                    if b == solid.len()
                        || (a < cached.solid.len() && cached.solid[a].depth >= solid[b].depth)
                    {
                        draw(&cached.solid[a], offset, &colors, window);
                        a += 1;
                    } else {
                        draw(&solid[b], offset, &colors, window);
                        b += 1;
                    }
                }
                if profile::record(
                    begin,
                    cached.floor.len() + floor.len() + cached.solid.len() + solid.len(),
                    g.board.key,
                    [key[2], key[3]],
                    window.scale_factor(),
                ) {
                    cx.defer(|cx| cx.quit());
                }
            });
        },
    )
    .size_full()
}
