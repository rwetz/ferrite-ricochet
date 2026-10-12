//! Live CPU paint/cadence measurements. GPU completion is explicitly outside this boundary.
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
    board: &'static str,
    viewport: [f32; 2],
    scale_factor: f32,
}
thread_local! {static PROFILE:RefCell<Profile>=RefCell::new(Profile::default());}
pub fn configure(path: PathBuf) {
    PROFILE.with_borrow_mut(|p| p.path = Some(path));
}
pub fn enabled() -> bool {
    PROFILE.with_borrow(|p| p.path.is_some())
}
fn stats(mut values: Vec<f64>) -> String {
    if values.is_empty() {
        return "null".into();
    }
    values.sort_by(f64::total_cmp);
    let at = |q: f64| values[((values.len() - 1) as f64 * q).round() as usize];
    format!(
        "{{\"p50_ms\":{:.4},\"p95_ms\":{:.4},\"p99_ms\":{:.4},\"max_ms\":{:.4}}}",
        at(0.5),
        at(0.95),
        at(0.99),
        values[values.len() - 1]
    )
}
pub fn record(
    start: Instant,
    draws: usize,
    board: &'static str,
    viewport: [f32; 2],
    scale_factor: f32,
) -> bool {
    PROFILE.with_borrow_mut(|p| {
  if p.done||p.path.is_none(){return false;}
  let now=Instant::now();let first=*p.start.get_or_insert(now);let elapsed=now.duration_since(first);
  p.board=board;p.viewport=viewport;p.scale_factor=scale_factor;
  if let Some(previous)=p.last.replace(now)&&elapsed>Duration::from_secs(2){
   p.intervals.push(now.duration_since(previous).as_secs_f64()*1000.);
   p.paint.push(now.duration_since(start).as_secs_f64()*1000.);p.draws=p.draws.max(draws);
  }
  if elapsed<Duration::from_secs(20){return false;}
  p.done=true;
  let mean=p.intervals.iter().sum::<f64>()/p.intervals.len().max(1)as f64;
  let outliers=p.intervals.iter().filter(|v|**v>20.).count();
  let output=format!(concat!("{{\"board\":\"{}\",\"viewport\":[{:.1},{:.1}],\"scale_factor\":{:.2},",
   "\"duration_s\":{:.3},\"warmup_s\":2,\"frames\":{},\"mean_observed_fps\":{:.3},",
   "\"frames_over_20ms\":{},\"max_path_draws\":{},\"cpu_paint\":{},\"paint_intervals\":{},",
   "\"limits\":\"App paint cadence and CPU callback time; excludes asynchronous GPU completion and GPU presentation timestamps.\"}}"),
   p.board,p.viewport[0],p.viewport[1],p.scale_factor,elapsed.as_secs_f64(),p.paint.len(),1000./mean,outliers,p.draws,stats(p.paint.clone()),stats(p.intervals.clone()));
  let path=p.path.as_ref().unwrap();if let Some(parent)=path.parent(){let _=std::fs::create_dir_all(parent);}
  if let Err(error)=std::fs::write(path,&output){eprintln!("Profile write failed: {error}");}else{println!("{output}");}
  true
 })
}
