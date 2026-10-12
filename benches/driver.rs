#[path = "../src/benchmark.rs"]
mod benchmark;
#[path = "../src/physics.rs"]
mod physics;
#[path = "../src/scene.rs"]
mod scene;
#[path="../src/board.rs"]mod board;
fn main() {
    if std::env::args().any(|a|a=="--snapshot-all"){benchmark::snapshot_all();}
    else if std::env::args().any(|a|a=="--smoke-all"){benchmark::smoke_all();}
    else if std::env::args().any(|a|a=="--all"){benchmark::all();}
    else if std::env::args().any(|a| a == "--snapshot") {
        benchmark::snapshot();
    } else {
        benchmark::run();
    }
}
