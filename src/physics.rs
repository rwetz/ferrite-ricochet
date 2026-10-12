use crate::board::{self, place};
use std::ops::{Add, Mul, Sub};
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V {
    pub x: f32,
    pub y: f32,
}
impl V {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn dot(self, b: Self) -> f32 {
        self.x * b.x + self.y * b.y
    }
    pub fn len(self) -> f32 {
        self.dot(self).sqrt()
    }
}
impl Add for V {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self::new(self.x + b.x, self.y + b.y)
    }
}
impl Sub for V {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        Self::new(self.x - b.x, self.y - b.y)
    }
}
impl Mul<f32> for V {
    type Output = Self;
    fn mul(self, b: f32) -> Self {
        Self::new(self.x * b, self.y * b)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RailKind {
    #[default]
    Orbit,
    Ramp,
}
#[derive(Clone)]
pub struct Game {
    pub board: &'static crate::board::Board,
    pub ball: V,
    pub vel: V,
    pub score: u64,
    pub best: u64,
    pub balls: u8,
    pub ball_limit: u8,
    pub mission: u8,
    pub overdrive: f32,
    pub rail_active: bool,
    pub rail_kind: RailKind,
    pub drops: [bool; 3],
    pub target_progress: u8,
    pub target_flash: [f32; 3],
    pub target_cooldown: [f32; 3],
    pub ramp_open: bool,
    pub ramp_runs: u32,
    pub jackpot_ready: bool,
    pub jackpots: u32,
    pub scoop_rewards_overdrive: bool,
    pub rail_distance: f32,
    pub rail_speed: f32,
    pub scoop_left: f32,
    pub completed_cycles: u32,
    pub live: bool,
    pub paused: bool,
    pub left: bool,
    pub right: bool,
    pub angles: [f32; 2],
    pub charge: f32,
    pub shaking: f32,
    pub flash: [f32; 3],
    pub slings: [f32; 2],
    pub trail: Vec<V>,
    pub time: f32,
    pub tilt: f32,
    pub message: &'static str,
    pub hits: u32,
    pub multiplier: u64,
    pub lamps: [bool; 3],
    pub rollovers: [bool; 3],
    pub lane_flash: [f32; 3],
    pub combo: u32,
    pub combo_left: f32,
    pub save_left: f32,
    pub score_parts: [u64; 4],
    pub recoveries: u32,
    pub(crate) stuck_anchor: V,
    pub(crate) stuck_time: f32,
}
impl Default for Game {
    fn default() -> Self {
        Self {
            board: &crate::board::FERRITE,
            ball: board::SHOOTER,
            vel: V::default(),
            score: 0,
            best: 0,
            balls: 3,
            ball_limit: 3,
            mission: 0,
            overdrive: 0.,
            rail_active: false,
            rail_kind: RailKind::Orbit,
            drops: [false; 3],
            target_progress: 0,
            target_flash: [0.; 3],
            target_cooldown: [0.; 3],
            ramp_open: false,
            ramp_runs: 0,
            jackpot_ready: false,
            jackpots: 0,
            scoop_rewards_overdrive: false,
            rail_distance: 0.,
            rail_speed: 0.,
            scoop_left: 0.,
            completed_cycles: 0,
            live: false,
            paused: false,
            left: false,
            right: false,
            angles: [0.42, std::f32::consts::PI - 0.42],
            charge: 0.,
            shaking: 0.,
            flash: [0.; 3],
            slings: [0.; 2],
            trail: Vec::new(),
            time: 0.,
            tilt: 0.,
            message: "HOLD SPACE TO CHARGE",
            hits: 0,
            multiplier: 1,
            lamps: [false; 3],
            rollovers: [false; 3],
            lane_flash: [0.; 3],
            combo: 0,
            combo_left: 0.,
            save_left: 0.,
            score_parts: [0; 4],
            recoveries: 0,
            stuck_anchor: board::SHOOTER,
            stuck_time: 0.,
        }
    }
}
impl Game {
    pub fn reset(&mut self) {
        let best = self.best;
        let limit = self.ball_limit;
        let board = self.board;
        *self = Self::default();
        self.best = best;
        self.ball_limit = limit;
        self.balls = limit;
        self.board = board;
    }
    pub fn set_board(&mut self, board: &'static crate::board::Board) {
        if self.board.key != board.key {
            self.board = board;
            self.reset();
        }
    }
    pub fn set_ball_limit(&mut self, limit: u8) {
        if matches!(limit, 3 | 5) {
            self.ball_limit = limit;
            self.reset();
        }
    }
    fn add_score(&mut self, base: u64, part: usize) {
        let points = base * self.multiplier * if self.overdrive > 0. { 2 } else { 1 };
        self.score += points;
        self.score_parts[part] += points;
    }
    pub fn active_rail(&self) -> &'static std::sync::LazyLock<crate::board::Rail> {
        match self.rail_kind {
            RailKind::Orbit => self.board.orbit,
            RailKind::Ramp => self.board.ramp,
        }
    }
    fn rail_step(&mut self, dt: f32) {
        let rail = self.active_rail();
        let (_, tangent) = rail.sample(self.rail_distance);
        let slope = rail.slope(self.rail_distance);
        self.rail_speed += (260. * tangent.y - 620. * slope) / (1. + slope * slope).sqrt() * dt;
        self.rail_speed *= 1. - dt * 0.06;
        self.rail_distance += self.rail_speed * dt;
        if self.rail_distance >= rail.length {
            let (p, t) = rail.sample(rail.length);
            self.ball = p + t * 8.;
            self.vel = t * self.rail_speed;
            self.rail_active = false;
            if self.rail_kind == RailKind::Ramp {
                self.ramp_runs += 1;
                self.add_score(1500 + 250 * self.ramp_runs as u64, 2);
                self.jackpot_ready = true;
                self.ramp_open = false;
                self.drops = [false; 3];
                self.target_progress = 0;
                self.shaking = 0.7;
                self.message = "CROSSOVER COMPLETE / JACKPOT LIT";
            } else if self.mission == 1 {
                self.mission = 2;
                self.add_score(1000, 2);
                self.shaking = 0.6;
                self.message = "ORBIT COMPLETE / REACTOR OPEN";
            }
            return;
        }
        if self.rail_distance < 0. {
            let (p, t) = rail.sample(0.);
            self.ball = p - t * 8.;
            self.vel = t * self.rail_speed;
            self.rail_active = false;
            self.message = "RAIL REJECTED / BUILD MOMENTUM";
            return;
        }
        let (p, t) = rail.sample(self.rail_distance);
        self.ball = p;
        self.vel = t * self.rail_speed;
    }
    fn target_hit(&mut self, i: usize) {
        self.target_cooldown[i] = 0.2;
        self.target_flash[i] = 1.;
        self.shaking = 0.35;
        if self.ramp_open {
            return;
        }
        if self.board.order[self.target_progress as usize] == i {
            self.drops[i] = true;
            self.target_progress += 1;
            self.add_score(150, 1);
            self.message = "TARGET DOWN / KEEP THE SEQUENCE";
            if self.target_progress == 3 {
                self.ramp_open = true;
                self.add_score(500, 1);
                self.message = "TARGET BANK COMPLETE / CROSSOVER OPEN";
            }
        } else {
            self.drops = [false; 3];
            self.target_progress = 0;
            self.target_flash[i] = -1.;
            self.message = "WRONG TARGET / BANK RESET";
        }
    }
    pub fn launch(&mut self) {
        if !self.live && self.balls > 0 && !self.paused {
            self.vel = V::new(-12., -445. - self.charge * 160.);
            self.live = true;
            self.charge = 0.;
            self.save_left = 6.;
            self.message = "BALL SAVE / 6 SECONDS";
        }
    }
    pub fn nudge(&mut self) {
        if self.live && !self.paused {
            self.tilt += 0.42;
            self.shaking = 1.;
            if self.tilt > 1. {
                self.message = "TILT / FLIPPERS DISABLED";
            } else {
                self.vel = self.vel
                    + V::new(
                        if self.ball.x < board::WIDTH / 2. {
                            65.
                        } else {
                            -65.
                        },
                        -85.,
                    );
                self.message = "NUDGE";
            }
        }
    }
    pub fn flipper(&self, i: usize) -> (V, V) {
        let a = board::flipper_pivot(i);
        (
            a,
            a + V::new(self.angles[i].cos(), self.angles[i].sin()) * board::FLIPPER_LENGTH,
        )
    }
    pub fn step(&mut self, dt: f32) {
        if self.paused {
            return;
        }
        self.time += dt;
        if self.overdrive > 0. {
            self.overdrive = (self.overdrive - dt).max(0.);
            if self.overdrive == 0. {
                self.mission = 0;
                self.lamps = [false; 3];
                self.message = "REACTOR COOLED / BREAK THE BANK";
            }
        }

        self.save_left = (self.save_left - dt).max(0.);
        self.combo_left = (self.combo_left - dt).max(0.);
        if self.combo_left <= 0. {
            self.combo = 0;
        }
        for f in &mut self.lane_flash {
            *f = (*f - dt * 2.).max(0.);
        }
        for f in &mut self.target_flash {
            *f = if *f < 0. {
                (*f + dt * 3.).min(0.)
            } else {
                (*f - dt * 3.).max(0.)
            };
        }
        for f in &mut self.target_cooldown {
            *f = (*f - dt).max(0.);
        }
        self.shaking = (self.shaking - dt * 4.).max(0.);
        for f in &mut self.flash {
            *f = (*f - dt * 3.).max(0.);
        }
        for cooldown in &mut self.slings {
            *cooldown = (*cooldown - dt).max(0.);
        }
        self.tilt = (self.tilt - dt * 0.05).max(0.);
        let old = self.angles;
        let enabled = self.tilt < 1.;
        let target = [
            if self.left && enabled { -0.52 } else { 0.42 },
            std::f32::consts::PI - if self.right && enabled { -0.52 } else { 0.42 },
        ];
        for (i, t) in target.iter().enumerate() {
            self.angles[i] += (*t - self.angles[i]).clamp(-dt * 14., dt * 14.);
        }
        if !self.live {
            return;
        }
        if self.scoop_left > 0. {
            self.scoop_left = (self.scoop_left - dt).max(0.);
            self.ball = self.board.scoop;
            if self.scoop_left == 0. {
                self.vel = V::new(50., 220.);
                if self.scoop_rewards_overdrive {
                    self.overdrive = 20.;
                    self.scoop_rewards_overdrive = false;
                }
            }
            return;
        }
        if self.rail_active {
            self.rail_step(dt);
            self.ball_search(dt);
            return;
        }
        self.vel.y += dt * 260.;
        self.vel = self.vel * (1. - dt * 0.07);
        let previous_ball = self.ball;
        self.ball = self.ball + self.vel * dt;
        if self.ball.x > 202. * board::SPACE_X
            && self.ball.y < 53. * board::SPACE_Y
            && self.vel.y < 0.
        {
            // Clear the diagonal shooter guide before redirecting. The authored
            // upper core determines the launch chute's angle on each machine.
            let aim = self
                .board
                .bumpers
                .iter()
                .min_by(|a, b| a.y.total_cmp(&b.y))
                .unwrap();
            let flight = ((self.ball.x - aim.x) / 215.).max(0.2);
            self.vel = V::new(
                -215.,
                (aim.y - self.ball.y - 130. * flight * flight) / flight,
            );
        }
        for i in 0..3 {
            if (previous_ball.y - self.board.lanes[i].y) * (self.ball.y - self.board.lanes[i].y)
                < 0.
                && (self.ball.x - self.board.lanes[i].x).abs() < 10.
                && self.lane_flash[i] <= 0.
            {
                self.rollovers[i] = true;
                self.lane_flash[i] = 1.;
                self.add_score(75, 0);
                self.message = "ROLLOVER / +75";
                if self.rollovers.iter().all(|v| *v) {
                    self.add_score(500, 1);
                    self.rollovers = [false; 3];
                    self.message = "LANES COMPLETE / +250";
                }
            }
        }
        let rail = self.board.orbit;
        let (entry, tangent) = rail.sample(0.);
        if self.mission >= 1 && (self.ball - entry).len() < 9. && self.vel.dot(tangent) > 80. {
            self.rail_active = true;
            self.rail_kind = RailKind::Orbit;
            self.rail_distance = 0.;
            self.rail_speed = self.vel.dot(tangent);
            self.ball = entry;
            self.message = "ORBIT / CARRY THE MOMENTUM";
            return;
        }
        let ramp = self.board.ramp;
        let (entry, tangent) = ramp.sample(0.);
        if self.ramp_open && (self.ball - entry).len() < 9. && self.vel.dot(tangent) > 80. {
            self.rail_active = true;
            self.rail_kind = RailKind::Ramp;
            self.rail_distance = 0.;
            self.rail_speed = self.vel.dot(tangent);
            self.ball = entry;
            self.message = "CROSSOVER / CLIMB THE CREST";
            return;
        }
        if !self.ramp_open {
            let (a, b) = self.board.ramp_gate;
            self.collide(a, b, 1.5, 0.75, V::default());
        }
        if self.mission == 0 {
            let (a, b) = self.board.gate;
            self.collide(a, b, 1.5, 0.75, V::default());
        }
        for (index, (a, b)) in self.board.walls.iter().copied().enumerate() {
            if index == 9 || index == 10 {
                let ab = b - a;
                let t = ((self.ball - a).dot(ab) / ab.dot(ab)).clamp(0., 1.);
                let d = self.ball - (a + ab * t);
                let l = d.len();
                let incoming = l > 0.001 && self.vel.dot(d * (1. / l)) < 0.;
                self.collide(a, b, 2., 0.95, V::default());
                let sling = index - 9;
                if l < 6. && incoming && self.slings[sling] <= 0. {
                    self.vel = self.vel + V::new(if sling == 0 { 95. } else { -95. }, -120.);
                    self.add_score(25, 0);
                    self.slings[sling] = 0.15;
                    self.shaking = 0.35;
                    self.message = "SLING / +25";
                }
            } else {
                self.collide(a, b, 2., 0.82, V::default());
            }
        }
        for i in 0..3 {
            if self.drops[i] {
                continue;
            }
            let target = self.board.targets[i];
            let side = V::new(target.normal.y, -target.normal.x);
            let a = target.center - side * 5.;
            let b = target.center + side * 5.;
            let ab = b - a;
            let t = ((self.ball - a).dot(ab) / ab.dot(ab)).clamp(0., 1.);
            let d = self.ball - (a + ab * t);
            let incoming = self.vel.dot(target.normal) < -10. && d.dot(target.normal) > 0.;
            self.collide(a, b, 1.5, 0.85, V::default());
            if d.len() < 5.5 && incoming && self.target_cooldown[i] <= 0. {
                self.target_hit(i);
            }
        }
        for (i, previous) in old.iter().enumerate() {
            let (a, b) = self.flipper(i);
            let arm = self.ball - a;
            let omega = (self.angles[i] - previous) / dt;
            self.collide(a, b, 3., 0.83, V::new(-arm.y * omega, arm.x * omega));
        }
        for (i, c) in self.board.bumpers.iter().enumerate() {
            let d = self.ball - *c;
            let l = d.len();
            if l < 18. && l > 0.001 {
                let n = d * (1. / l);
                self.ball = *c + n * 18.;
                if self.vel.dot(n) < 0. {
                    self.vel = self.vel - n * (2. * self.vel.dot(n)) + n * 105.;
                    self.combo = (self.combo + 1).min(5);
                    self.combo_left = 2.5;
                    self.add_score(100 * self.combo as u64, 0);
                    self.hits += 1;
                    self.flash[i] = 1.;
                    self.shaking = 0.65;
                    self.lamps[i] = true;
                    self.message = "BUMPER / +100";
                    if self.lamps.iter().all(|x| *x) {
                        self.add_score(750, 1);
                        self.multiplier = (self.multiplier + 1).min(5);
                        self.lamps = [false; 3];
                        self.message = "BANK COMPLETE / MULTIPLIER UP";
                        if self.mission == 0 {
                            self.mission = 1;
                            self.message = "BANK COMPLETE / ORBIT GATE OPEN";
                        }
                    }
                }
            }
        }
        if self.jackpot_ready && (self.ball - self.board.scoop).len() < 9. {
            self.add_score(5000 + 1000 * self.ramp_runs as u64, 3);
            self.jackpots += 1;
            self.jackpot_ready = false;
            self.scoop_left = 0.6;
            self.scoop_rewards_overdrive = false;
            self.vel = V::default();
            self.shaking = 1.;
            self.message = "JACKPOT / REACTOR CAPTURE";
        }
        if self.mission == 2 && (self.ball - self.board.scoop).len() < 9. {
            self.mission = 3;
            self.scoop_left = 0.8;
            self.scoop_rewards_overdrive = true;
            self.completed_cycles += 1;
            self.add_score(2000, 3);
            self.vel = V::default();
            self.shaking = 0.8;
            self.message = "REACTOR CAPTURE / OVERDRIVE";
        }
        let speed = self.vel.len();
        if speed > 720. {
            self.vel = self.vel * (720. / speed);
        }
        self.best = self.best.max(self.score);
        self.ball_search(dt);
        if self.ball.y > board::DRAIN_Y && self.save_left > 0. {
            self.ball = board::SHOOTER;
            self.vel = V::new(-12., -560.);
            self.save_left = 0.;
            self.trail.clear();
            self.message = "BALL SAVED / BACK IN PLAY";
            return;
        }
        if self.ball.y > board::DRAIN_Y {
            self.combo = 0;
            self.combo_left = 0.;
            self.balls = self.balls.saturating_sub(1);
            self.live = false;
            self.ball = board::SHOOTER;
            self.vel = V::default();
            self.trail.clear();
            self.tilt = 0.;
            self.message = if self.balls == 0 {
                "GAME OVER / R TO RESTART"
            } else {
                "DRAIN / HOLD SPACE"
            };
        }
    }
    fn ball_search(&mut self, dt: f32) {
        // Monitor spatial progress, not speed. A ball can bounce quickly inside
        // a tiny pocket forever. Normal captures/rail travel bypass this code.
        if (self.ball - self.stuck_anchor).len() > 12. {
            self.stuck_anchor = self.ball;
            self.stuck_time = 0.;
        } else {
            self.stuck_time += dt;
        }
        if self.stuck_time > 3. {
            let left = self.ball.x < board::WIDTH / 2.;
            // Recover onto the apron-side open field, above the flippers and
            // below all map-specific targets. No score or ball is consumed.
            self.ball = place(V::new(if left { 92. } else { 134. }, 282.));
            self.vel = V::new(if left { 40. } else { -40. }, 80.);
            self.stuck_anchor = self.ball;
            self.stuck_time = 0.;
            self.recoveries += 1;
            self.rail_active = false;
            self.trail.clear();
            self.message = "BALL SEARCH / BACK IN PLAY";
        }
    }
    fn collide(&mut self, a: V, b: V, r: f32, restitution: f32, surface: V) {
        let ab = b - a;
        let t = ((self.ball - a).dot(ab) / ab.dot(ab)).clamp(0., 1.);
        let nearest = a + ab * t;
        let d = self.ball - nearest;
        let l = d.len();
        if l < r + 4. && l > 0.0001 {
            let n = d * (1. / l);
            self.ball = nearest + n * (r + 4.);
            let incoming = (self.vel - surface).dot(n);
            if incoming < 0. {
                self.vel = self.vel - n * ((1. + restitution) * incoming);
            }
        }
    }
    pub fn tick(&mut self, dt: f32, charging: bool) {
        let dt = dt.min(0.05);
        let steps = (dt / (1. / 240.)).ceil().max(1.) as usize;
        for _ in 0..steps {
            self.step(dt / steps as f32);
        }
        if charging && !self.live && !self.paused {
            self.charge = (self.charge + dt * 0.8).min(1.);
        }
        self.best = self.best.max(self.score);
        if self.live {
            self.trail.push(self.ball);
            if self.trail.len() > 12 {
                self.trail.remove(0);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reported_switchyard_gate_pocket_now_releases_naturally() {
        let start = place(V::new(28., 244.));
        let mut g = Game {
            board: &board::CONCRETE,
            live: true,
            ball: start,
            save_left: 0.,
            ..Game::default()
        };
        for _ in 0..480 {
            g.step(1. / 240.);
        }
        assert!((g.ball - start).len() > 30. || !g.live);
        assert_eq!(
            g.recoveries, 0,
            "Known pocket should be removed, not hidden by ball search"
        );
    }
    #[test]
    fn ball_search_recovers_a_spatially_trapped_fast_bounce_without_awards() {
        let mut g = Game {
            live: true,
            ball: place(V::new(45., 240.)),
            score: 1234,
            ..Game::default()
        };
        g.stuck_anchor = g.ball;
        for i in 0..800 {
            g.ball = g.stuck_anchor + V::new(if i % 2 == 0 { 2. } else { -2. }, 0.);
            g.vel = V::new(200., 0.);
            g.ball_search(1. / 240.);
            if g.recoveries > 0 {
                break;
            }
        }
        assert_eq!(g.recoveries, 1);
        assert_eq!(g.score, 1234);
        assert_eq!(g.balls, 3);
        assert_eq!(g.ball, place(V::new(92., 282.)));
    }
    #[test]
    fn captures_and_pauses_do_not_trigger_ball_search() {
        let mut g = Game {
            live: true,
            paused: true,
            stuck_time: 2.99,
            ..Game::default()
        };
        for _ in 0..400 {
            g.step(1. / 240.);
        }
        assert_eq!(g.recoveries, 0);
        g.paused = false;
        g.scoop_left = 0.8;
        g.ball = g.board.scoop;
        for _ in 0..100 {
            g.step(1. / 240.);
        }
        assert_eq!(g.recoveries, 0);
    }
    #[test]
    fn score_breakdown_includes_multipliers_and_sums_to_total() {
        let mut g = Game {
            multiplier: 3,
            overdrive: 10.,
            ..Game::default()
        };
        g.add_score(100, 0);
        g.add_score(150, 1);
        g.add_score(1000, 2);
        g.add_score(5000, 3);
        assert_eq!(g.score, 37500);
        assert_eq!(g.score, g.score_parts.iter().sum::<u64>());
    }
    #[test]
    fn launched_ball_survives_and_scores() {
        let mut g = Game {
            charge: 1.,
            ..Game::default()
        };
        g.launch();
        for i in 0..2400 {
            g.left = i % 60 < 30;
            g.right = i % 73 < 35;
            g.tick(1. / 120., false);
            assert!(g.ball.x.is_finite() && g.ball.y.is_finite());
        }
        assert!(g.hits > 0, "launch must reach scoring bank");
    }
    #[test]
    fn drains_end_after_three_balls() {
        let mut g = Game::default();
        for _ in 0..3 {
            g.launch();
            g.save_left = 0.;
            g.ball = place(V::new(112., 380.));
            g.step(1. / 240.);
        }
        assert_eq!(g.balls, 0);
        g.launch();
        assert!(!g.live);
    }
    #[test]
    fn pause_freezes_and_reset_preserves_best() {
        let mut g = Game {
            paused: true,
            best: 900,
            ..Game::default()
        };
        g.launch();
        g.tick(0.03, true);
        assert_eq!(g.charge, 0.);
        g.reset();
        assert_eq!(g.best, 900);
        assert_eq!(g.balls, 3);
    }
    #[test]
    fn flipper_strikes_upward() {
        let mut g = Game {
            live: true,
            left: true,
            ball: place(V::new(94., 312.)),
            vel: V::new(0., 40.),
            ..Game::default()
        };
        for _ in 0..12 {
            g.step(1. / 240.);
        }
        assert!(g.vel.y < 0., "rising flipper must transfer velocity");
    }
    #[test]
    fn launch_lane_cannot_award_sling_points() {
        let mut g = Game {
            live: true,
            ball: place(V::new(212., 245.)),
            vel: V::new(0., -80.),
            ..Game::default()
        };
        g.step(1. / 240.);
        assert_eq!(g.score, 0);
    }
    #[test]
    fn quick_plunge_reaches_playfield() {
        let mut g = Game::default();
        g.launch();
        let mut entered = false;
        for _ in 0..400 {
            g.tick(1. / 120., false);
            entered |= g.live && g.ball.x < 195. * board::SPACE_X;
        }
        assert!(entered, "even a quick press must feed the table");
    }
    #[test]
    fn early_drain_saves_once() {
        let mut g = Game::default();
        g.launch();
        g.ball = place(V::new(112., 380.));
        g.step(1. / 240.);
        assert_eq!(g.balls, 3);
        assert!(g.live);
        assert_eq!(g.save_left, 0.);
        g.ball = place(V::new(112., 380.));
        g.step(1. / 240.);
        assert_eq!(g.balls, 2);
    }
    #[test]
    fn rollover_crossing_scores_once() {
        let mut g = Game {
            live: true,
            ball: place(V::new(107., 53.8)),
            vel: V::new(0., 100.),
            ..Game::default()
        };
        g.step(1. / 240.);
        assert_eq!(g.score, 75);
        assert!(g.rollovers[1]);
        g.step(1. / 240.);
        assert_eq!(g.score, 75);
    }
    #[test]
    fn combo_expires_without_a_hit() {
        let mut g = Game {
            combo: 3,
            combo_left: 0.01,
            ..Game::default()
        };
        g.tick(0.02, false);
        assert_eq!(g.combo, 0);
    }
    #[test]
    fn fast_orbit_unlocks_reactor() {
        let mut g = Game {
            live: true,
            mission: 1,
            rail_active: true,
            rail_speed: 550.,
            ..Game::default()
        };
        for _ in 0..1200 {
            g.step(1. / 240.);
            if !g.rail_active {
                break;
            }
        }
        assert_eq!(g.mission, 2);
        assert!(g.score >= 750);
    }
    #[test]
    fn weak_orbit_rolls_back() {
        let mut g = Game {
            live: true,
            mission: 1,
            rail_active: true,
            rail_speed: 110.,
            ..Game::default()
        };
        for _ in 0..1200 {
            g.step(1. / 240.);
            if !g.rail_active {
                break;
            }
        }
        assert!(!g.rail_active);
        assert_eq!(g.mission, 1);
        assert_eq!(g.score, 0);
    }
    #[test]
    fn reactor_requires_sequence() {
        let mut g = Game {
            live: true,
            mission: 2,
            ball: crate::board::FERRITE.scoop,
            ..Game::default()
        };
        g.step(1. / 240.);
        assert_eq!(g.mission, 3);
        assert!(g.scoop_left > 0.);
        for _ in 0..220 {
            g.step(1. / 240.);
        }
        assert!(g.overdrive > 19.);
    }
    #[test]
    fn five_ball_round_survives_reset() {
        let mut g = Game::default();
        g.set_ball_limit(5);
        g.balls = 2;
        g.reset();
        assert_eq!(g.balls, 5);
    }
    #[test]
    fn ordered_targets_open_ramp_and_wrong_shots_reset() {
        for board in crate::board::ALL {
            let mut g = Game {
                board,
                ..Game::default()
            };
            g.target_hit(board.order[0]);
            assert_eq!(g.target_progress, 1);
            g.target_hit(board.order[2]);
            assert_eq!(g.target_progress, 0);
            assert_eq!(g.drops, [false; 3]);
            for i in board.order {
                g.target_hit(i);
            }
            assert!(g.ramp_open);
            assert_eq!(g.drops, [true; 3]);
        }
    }
    #[test]
    fn target_requires_actual_front_contact() {
        let board = &crate::board::FERRITE;
        let p = board.targets[board.order[0]].center;
        let mut g = Game {
            live: true,
            ball: p + V::new(0., 4.8),
            vel: V::new(0., -150.),
            ..Game::default()
        };
        g.step(1. / 240.);
        assert_eq!(g.target_progress, 1);
    }
    #[test]
    fn ramps_require_inertia_and_light_jackpot() {
        for board in crate::board::ALL {
            let mut g = Game {
                board,
                live: true,
                ramp_open: true,
                rail_active: true,
                rail_kind: RailKind::Ramp,
                rail_speed: 590.,
                ..Game::default()
            };
            for _ in 0..1800 {
                g.step(1. / 240.);
                if !g.rail_active {
                    break;
                }
            }
            assert!(!g.rail_active, "{}: rail trapped a fast shot", board.key);
            assert!(
                g.jackpot_ready,
                "{}: full-speed shot cannot finish",
                board.key
            );
            assert_eq!(g.ramp_runs, 1);
            let mut slow = Game {
                board,
                live: true,
                ramp_open: true,
                rail_active: true,
                rail_kind: RailKind::Ramp,
                rail_speed: 100.,
                ..Game::default()
            };
            for _ in 0..1200 {
                slow.step(1. / 240.);
                if !slow.rail_active {
                    break;
                }
            }
            assert!(!slow.jackpot_ready);
            assert_eq!(slow.ramp_runs, 0);
        }
    }
    #[test]
    fn jackpot_capture_does_not_bypass_overdrive_sequence() {
        let mut g = Game {
            live: true,
            jackpot_ready: true,
            ramp_runs: 1,
            ball: crate::board::FERRITE.scoop,
            ..Game::default()
        };
        g.step(1. / 240.);
        assert_eq!(g.jackpots, 1);
        assert!(g.scoop_left > 0.);
        for _ in 0..200 {
            g.step(1. / 240.);
        }
        assert_eq!(g.overdrive, 0.);
        assert_eq!(g.mission, 0);
    }
    #[test]
    fn switch_resets_round_and_preserves_selected_limit_and_best() {
        let mut g = Game {
            score: 900,
            best: 1200,
            ball_limit: 5,
            mission: 2,
            ramp_open: true,
            ..Game::default()
        };
        g.set_board(&crate::board::MONO);
        assert_eq!(g.board.key, "mono");
        assert_eq!(g.score, 0);
        assert_eq!(g.balls, 5);
        assert_eq!(g.best, 1200);
        assert_eq!(g.mission, 0);
        assert!(!g.ramp_open);
    }
}
