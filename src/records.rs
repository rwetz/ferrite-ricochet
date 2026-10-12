//! Each authored map owns its record. The original record migrates only to Ferrite.
use crate::board::Board;
use std::path::PathBuf;
fn root() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share")))
    };
    base.map(|p| p.join("ferrite-ricochet"))
}
fn read(path: PathBuf) -> Option<u64> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}
pub fn load(board: &Board) -> u64 {
    let Some(root) = root() else {
        return 0;
    };
    read(root.join("records").join(format!("{}.txt", board.key)))
        .or_else(|| {
            if board.key == "ferrite" {
                read(root.join("high-score.txt"))
            } else {
                None
            }
        })
        .unwrap_or(0)
}
pub fn save(board: &Board, best: u64) {
    let Some(root) = root() else {
        return;
    };
    let path = root.join("records").join(format!("{}.txt", board.key));
    let best = best.max(read(path.clone()).unwrap_or(0));
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, best.to_string());
}
