//! Live measurements distinguish CPU paint time from presented frame intervals.
use std::{
    cell::RefCell,
    path::PathBuf,
    time::{Duration, Instant},
};
#[derive(Default)]
struct Profile {
    path: Option<PathBuf>,
    start: Option<Instant>,
    last: Option<Instant>,
    paint: Vec<f64>,
    intervals: Vec<f64>,
    draws: usize,
    done: bool,
}
thread_local! {static PROFILE:RefCell<Profile>=RefCell::new(Profile::default());}
pub fn configure(path: PathBuf) {
    PROFILE.with_borrow_mut(|p| p.path = Some(path));
}
pub fn enabled() -> bool {
    PROFILE.with_borrow(|p| p.path.is_some())
}
fn stats(mut v: Vec<f64>) -> String {
    if v.is_empty() {
        return "null".into();
    }
    v.sort_by(f64::total_cmp);
    let at = |q: f64| v[((v.len() - 1) as f64 * q).round() as usize];
    format!(
        "{{\"p50_ms\":{:.4},\"p95_ms\":{:.4},\"p99_ms\":{:.4},\"max_ms\":{:.4}}}",
        at(0.5),
        at(0.95),
        at(0.99),
        v[v.len() - 1]
    )
}
pub fn record(start: Instant, draws: usize) -> bool {
    PROFILE.with_borrow_mut(|p|{if p.done||p.path.is_none(){return false;}let now=Instant::now();let first=*p.start.get_or_insert(now);let elapsed=now.duration_since(first);if let Some(previous)=p.last.replace(now) && elapsed>Duration::from_secs(2){p.intervals.push(now.duration_since(previous).as_secs_f64()*1000.);p.paint.push(now.duration_since(start).as_secs_f64()*1000.);p.draws=p.draws.max(draws);}
 if elapsed<Duration::from_secs(20){return false;}p.done=true;let mean=p.intervals.iter().sum::<f64>()/p.intervals.len().max(1)as f64;let over=p.intervals.iter().filter(|t|**t>20.).count();let out=format!("{{\"duration_s\":{:.3},\"warmup_s\":2,\"frames\":{},\"mean_observed_fps\":{:.3},\"frames_over_20ms\":{},\"max_path_draws\":{},\"cpu_paint\":{},\"paint_intervals\":{},\"limits\":\"Paint intervals are app frame cadence, not GPU presentation timestamps. CPU paint includes projection and submission but not asynchronous GPU completion.\"}}",elapsed.as_secs_f64(),p.paint.len(),1000./mean,over,p.draws,stats(p.paint.clone()),stats(p.intervals.clone()));let path=p.path.as_ref().unwrap();if let Some(parent)=path.parent(){let _=std::fs::create_dir_all(parent);}
 if let Err(error)=std::fs::write(path,&out){eprintln!("Profile write failed: {error}");}else{println!("{out}");}true
})
}
