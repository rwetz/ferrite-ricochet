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
