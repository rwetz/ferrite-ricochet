//! Completed rounds rank separately by board and ball limit. Legacy records
//! remain readable without mixing the old scoring rules into V5 rankings.
use crate::{board::Board, physics::Game};
use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub score: u64,
    pub hits: u32,
    pub circuits: u32,
    pub ramps: u32,
    pub jackpots: u32,
    pub id: u128,
}
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
pub fn legacy_best(board: &Board) -> u64 {
    let read = |path: PathBuf| {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
    };
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
fn filename(board: &Board, balls: u8) -> String {
    format!("{}-{}-balls.txt", board.key, balls)
}
fn sort(rows: &mut Vec<Entry>) {
    rows.sort_by(|a, b| b.score.cmp(&a.score).then(b.id.cmp(&a.id)));
    rows.truncate(5);
}
fn parse(s: &str) -> Vec<Entry> {
    let mut rows = s
        .lines()
        .filter_map(|line| {
            let f = line.split('|').collect::<Vec<_>>();
            if f.len() != 6 {
                return None;
            }
            let e = Entry {
                score: f[0].parse().ok()?,
                hits: f[1].parse().ok()?,
                circuits: f[2].parse().ok()?,
                ramps: f[3].parse().ok()?,
                jackpots: f[4].parse().ok()?,
                id: f[5].parse().ok()?,
            };
            (e.score > 0).then_some(e)
        })
        .collect::<Vec<_>>();
    sort(&mut rows);
    rows
}
fn read(path: &Path) -> Vec<Entry> {
    std::fs::read_to_string(path)
        .map(|s| parse(&s))
        .unwrap_or_default()
}
pub fn top(board: &Board, balls: u8) -> Vec<Entry> {
    root()
        .map(|r| read(&r.join("leaderboards-v5").join(filename(board, balls))))
        .unwrap_or_default()
}
fn persist(path: &Path, entry: Entry) -> Result<Option<usize>, String> {
    if entry.score == 0 {
        return Ok(None);
    }
    let id = entry.id;
    let mut rows = read(path);
    rows.push(entry);
    sort(&mut rows);
    let rank = rows.iter().position(|e| e.id == id).map(|i| i + 1);
    let text = rows
        .iter()
        .map(|e| {
            format!(
                "{}|{}|{}|{}|{}|{}\n",
                e.score, e.hits, e.circuits, e.ramps, e.jackpots, e.id
            )
        })
        .collect::<String>();
    std::fs::create_dir_all(path.parent().ok_or("Missing score directory")?)
        .map_err(|e| e.to_string())?;
    let tmp = path.with_extension(format!("{}.tmp", std::process::id()));
    std::fs::write(&tmp, text)
        .and_then(|_| std::fs::rename(&tmp, path))
        .map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            e.to_string()
        })?;
    Ok(rank)
}
pub fn finish(g: &Game) -> Result<Option<usize>, String> {
    if g.balls != 0 || !matches!(g.ball_limit, 3 | 5) {
        return Err("Only completed rounds can rank".into());
    }
    let r = root().ok_or("Local score storage unavailable")?;
    persist(
        &r.join("leaderboards-v5")
            .join(filename(g.board, g.ball_limit)),
        Entry {
            score: g.score,
            hits: g.hits,
            circuits: g.completed_cycles,
            ramps: g.ramp_runs,
            jackpots: g.jackpots,
            id: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        },
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_rows_and_zero_scores_are_ignored() {
        let r = parse("bad\n100|2|0|1|0|1\n300|5|1|2|1|3\n0|0|0|0|0|2");
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].score, 300);
    }
    #[test]
    fn board_and_round_modes_rank_separately() {
        assert_ne!(
            filename(&crate::board::FERRITE, 3),
            filename(&crate::board::FERRITE, 5)
        );
        assert_ne!(
            filename(&crate::board::FERRITE, 3),
            filename(&crate::board::MONO, 3)
        );
    }
    #[test]
    fn unfinished_rounds_cannot_rank() {
        assert!(finish(&Game::default()).is_err());
    }
    #[test]
    fn atomic_replacement_preserves_five_best_scores() {
        let folder = std::env::temp_dir().join(format!(
            "ricochet-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = folder.join("scores.txt");
        for i in 1..8 {
            assert_eq!(
                persist(
                    &path,
                    Entry {
                        score: i * 100,
                        hits: 1,
                        circuits: 0,
                        ramps: 0,
                        jackpots: 0,
                        id: i as u128
                    }
                )
                .unwrap(),
                Some(1)
            );
        }
        let r = read(&path);
        assert_eq!(r.len(), 5);
        assert_eq!(r[0].score, 700);
        assert_eq!(r[4].score, 300);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(folder).unwrap();
    }
}
