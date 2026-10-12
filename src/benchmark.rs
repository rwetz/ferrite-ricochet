//! Stable CPU workloads. Native raster baseline excludes its additional resize/upload costs.
use crate::{
    physics::Game,
    scene::{self, Camera},
};
use std::{hint::black_box, time::Instant};
#[path = "../benches/legacy_raster.rs"]
mod legacy;
fn percentile(values: &mut [f64], q: f64) -> f64 {
    values.sort_by(f64::total_cmp);
    values[((values.len() - 1) as f64 * q).round() as usize]
}
fn stats(mut values: Vec<f64>) -> String {
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let p50 = percentile(&mut values, 0.5);
    let p95 = percentile(&mut values, 0.95);
    let p99 = percentile(&mut values, 0.99);
    format!("{{\"mean_ms\":{mean:.6},\"p50_ms\":{p50:.6},\"p95_ms\":{p95:.6},\"p99_ms\":{p99:.6}}}")
}
pub fn run() {
    let camera = Camera::new(900., 900.);
    let start = Instant::now();
    let static_faces = scene::static_table(&crate::board::FERRITE);
    let projected = scene::project_sorted(&camera, &static_faces);
    let static_ms = start.elapsed().as_secs_f64() * 1000.;
    black_box(&projected);
    let mut g = Game::default();
    let mut frames = Vec::with_capacity(420);
    let mut physics = Vec::new();
    for i in 0..420 {
        if !g.live {
            if g.balls == 0 {
                g.reset();
            }
            g.charge = 0.6;
            g.launch();
        }
        g.left = i % 43 < 17;
        g.right = i % 57 < 22;
        let t = Instant::now();
        g.tick(1. / 60., false);
        physics.push(t.elapsed().as_secs_f64() * 1000.);
        frames.push(g.clone());
    }
    let mut old = Vec::new();
    let mut new = Vec::new();
    let mut max_faces = 0;
    for (i, g) in frames.iter().enumerate() {
        let start = Instant::now();
        black_box(legacy::raster(black_box(g), true));
        let elapsed = start.elapsed().as_secs_f64() * 1000.;
        if i >= 60 {
            old.push(elapsed);
        }
        let start = Instant::now();
        let faces = scene::dynamic_table(black_box(g), true);
        let dynamic = scene::project_sorted(&camera, &faces);
        max_faces = max_faces.max(dynamic.len());
        black_box(dynamic);
        let elapsed = start.elapsed().as_secs_f64() * 1000.;
        if i >= 60 {
            new.push(elapsed);
        }
    }
    println!(
        "{{\"workload\":\"same 360 replay frames after 60 warmup frames; effects on; release optimization\",\"excludes\":\"GPUI tessellation, GPU submission/present; old resize and texture upload\",\"static_faces\":{},\"max_dynamic_faces\":{},\"static_build_ms\":{:.6},\"baseline_native_raster\":{},\"candidate_dynamic_mesh_projection\":{},\"physics_tick\":{}}}",
        static_faces.len(),
        max_faces,
        static_ms,
        stats(old),
        stats(new),
        stats(physics)
    );
}
pub fn snapshot() {
    let c = Camera::new(900., 1050.);
    let g = Game {
        ball: crate::physics::V::new(132., 232.),
        left: true,
        lamps: [true, false, true],
        charge: 0.6,
        ..Game::default()
    };
    let mut faces = scene::static_table(&crate::board::FERRITE);
    faces.extend(scene::dynamic_table(&g, true));
    let projected = scene::project_sorted(&c, &faces);
    println!("[");
    for (i, f) in projected.iter().enumerate() {
        let pts = f
            .points
            .iter()
            .map(|p| format!("[{:.3},{:.3}]", p[0], p[1]))
            .collect::<Vec<_>>()
            .join(",");
        println!(
            "{{\"ink\":{},\"points\":[{}]}}{}",
            f.ink,
            pts,
            if i + 1 < projected.len() { "," } else { "" }
        );
    }
    println!("]");
}

// Replay every authored board without opening a window. This is run on packaged binaries.
pub fn smoke_all() {
    for board in crate::board::ALL {
        let mut g = Game {
            board,
            ..Game::default()
        };
        let mut hits = 0;
        let mut launches = 0;
        for _ in 0..3600 {
            if !g.live {
                if g.balls == 0 {
                    g.reset();
                }
                g.charge = [0.2, 0.6, 1.][launches % 3];
                g.launch();
                launches += 1;
            }
            g.left = (g.time * 2.3).sin() > 0.2;
            g.right = (g.time * 1.9).cos() > 0.1;
            let before = g.hits;
            g.tick(1. / 60., false);
            hits += g.hits - before;
            assert!(
                g.ball.x.is_finite() && g.ball.y.is_finite(),
                "{}: non-finite ball",
                board.key
            );
        }
        assert!(
            hits > 0,
            "{}: launch replay never reached a bumper",
            board.key
        );
        println!(
            "{} / {}: {} bumper hits, {} launches",
            board.key, board.title, hits, launches
        );
    }
}
pub fn snapshot_all() {
    for board in crate::board::ALL {
        let c = Camera::new(600., 720.);
        let g = Game {
            board,
            ball: crate::physics::V::new(132., 282.),
            left: true,
            lamps: [true, false, true],
            charge: 0.6,
            ..Game::default()
        };
        let mut f = scene::static_table(board);
        f.extend(scene::dynamic_table(&g, true));
        let projected = scene::project_sorted(&c, &f);
        let faces = projected
            .iter()
            .map(|f| {
                format!(
                    "{{\"ink\":{},\"points\":[{}]}}",
                    f.ink,
                    f.points
                        .iter()
                        .map(|p| format!("[{:.3},{:.3}]", p[0], p[1]))
                        .collect::<Vec<_>>()
                        .join(",")
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        println!(
            "{{\"key\":\"{}\",\"title\":\"{}\",\"faces\":[{}]}}",
            board.key, board.title, faces
        );
    }
}
pub fn all() {
    println!("[");
    for (index, board) in crate::board::ALL.into_iter().enumerate() {
        let camera = Camera::new(900., 900.);
        let start = Instant::now();
        let fixed = scene::project_sorted(&camera, &scene::static_table(board));
        let build = start.elapsed().as_secs_f64() * 1000.;
        let mut g = Game {
            board,
            live: true,
            lamps: [true, false, true],
            ramp_open: true,
            ..Game::default()
        };
        let mut prepare = Vec::new();
        let mut max = 0;
        for i in 0..420 {
            g.ball = crate::physics::V::new(40. + (i % 160) as f32, 95. + (i % 185) as f32);
            g.left = i % 2 == 0;
            g.right = i % 3 == 0;
            g.rail_active = i % 4 == 0;
            g.rail_kind = crate::physics::RailKind::Ramp;
            g.rail_distance = (i as f32 / 420.) * g.board.ramp.length;
            g.flash[i % 3] = (i % 20) as f32 / 20.;
            let start = Instant::now();
            let f = scene::project_sorted(&camera, &scene::dynamic_table(black_box(&g), true));
            max = max.max(f.len());
            black_box(f);
            if i >= 60 {
                prepare.push(start.elapsed().as_secs_f64() * 1000.);
            }
        }
        println!(
            "{{\"key\":\"{}\",\"static_faces\":{},\"max_dynamic_faces\":{},\"build_ms\":{:.5},\"dynamic_prepare\":{}}}{}",
            board.key,
            fixed.len(),
            max,
            build,
            stats(prepare),
            if index < 9 { "," } else { "" }
        );
    }
    println!("]");
}
