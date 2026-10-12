#[path = "../src/benchmark.rs"]
mod benchmark;
#[path = "../src/physics.rs"]
mod physics;
#[path = "../src/scene.rs"]
mod scene;
fn main() {
    if std::env::args().any(|a| a == "--snapshot") {
        benchmark::snapshot();
    } else {
        benchmark::run();
    }
}
