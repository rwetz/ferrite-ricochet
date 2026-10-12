use std::ops::{Add, Mul, Sub};
#[derive(Clone, Copy, Debug, Default)]
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
pub const BUMPERS: [V; 3] = [V::new(78., 112.), V::new(150., 96.), V::new(120., 161.)];
pub const WALLS: [(V, V); 12] = [
    (V::new(22., 330.), V::new(22., 55.)),
    (V::new(22., 55.), V::new(48., 22.)),
    (V::new(48., 22.), V::new(185., 22.)),
    (V::new(185., 22.), V::new(221., 55.)),
    (V::new(221., 55.), V::new(221., 353.)),
    (V::new(202., 350.), V::new(202., 91.)),
    (V::new(202., 91.), V::new(182., 65.)),
    (V::new(22., 248.), V::new(68., 307.)),
    (V::new(202., 248.), V::new(158., 307.)),
    (V::new(43., 220.), V::new(73., 257.)),
    (V::new(181., 220.), V::new(153., 257.)),
    (V::new(22., 330.), V::new(66., 356.)),
];
#[derive(Clone)]
pub struct Game {
    pub ball: V,
    pub vel: V,
    pub score: u64,
    pub best: u64,
    pub balls: u8,
    pub live: bool,
    pub paused: bool,
    pub left: bool,
    pub right: bool,
    pub angles: [f32; 2],
    pub charge: f32,
    pub shaking: f32,
    pub flash: [f32; 3],
    pub trail: Vec<V>,
    pub time: f32,
    pub tilt: f32,
    pub message: &'static str,
    pub hits: u32,
    pub multiplier: u64,
    pub lamps: [bool; 3],
}
impl Default for Game {
    fn default() -> Self {
        Self {
            ball: V::new(212., 329.),
            vel: V::default(),
            score: 0,
            best: 0,
            balls: 3,
            live: false,
            paused: false,
            left: false,
            right: false,
            angles: [0.42, std::f32::consts::PI - 0.42],
            charge: 0.,
            shaking: 0.,
            flash: [0.; 3],
            trail: Vec::new(),
            time: 0.,
            tilt: 0.,
            message: "HOLD SPACE TO CHARGE",
            hits: 0,
            multiplier: 1,
            lamps: [false; 3],
        }
    }
}
impl Game {
    pub fn reset(&mut self) {
        let best = self.best;
        *self = Self::default();
        self.best = best;
    }
    pub fn launch(&mut self) {
        if !self.live && self.balls > 0 && !self.paused {
            self.vel = V::new(-12., -370. - self.charge * 170.);
            self.live = true;
            self.charge = 0.;
            self.message = "SHOOT THE BUMPER BANK";
        }
    }
    pub fn nudge(&mut self) {
        if self.live && !self.paused {
            self.tilt += 0.42;
            self.shaking = 1.;
            if self.tilt > 1. {
                self.message = "TILT / FLIPPERS DISABLED";
            } else {
                self.vel = self.vel + V::new(if self.ball.x < 112. { 65. } else { -65. }, -85.);
                self.message = "NUDGE";
            }
        }
    }
    pub fn flipper(&self, i: usize) -> (V, V) {
        let a = V::new(if i == 0 { 68. } else { 158. }, 307.);
        (
            a,
            a + V::new(self.angles[i].cos(), self.angles[i].sin()) * 37.,
        )
    }
    pub fn step(&mut self, dt: f32) {
        if self.paused {
            return;
        }
        self.time += dt;
        self.shaking = (self.shaking - dt * 4.).max(0.);
        for f in &mut self.flash {
            *f = (*f - dt * 3.).max(0.);
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
        self.vel.y += dt * 260.;
        self.vel = self.vel * (1. - dt * 0.07);
        self.ball = self.ball + self.vel * dt;
        if self.ball.x > 202. && self.ball.y < 68. && self.vel.y < 0. {
            self.ball = V::new(190., 58.);
            self.vel = V::new(-180., -75.);
        }
        for (a, b) in WALLS {
            self.collide(a, b, 2., 0.82, V::default());
        }
        for i in 0..2 {
            let (a, b) = self.flipper(i);
            let arm = self.ball - a;
            let omega = (self.angles[i] - old[i]) / dt;
            self.collide(a, b, 3., 0.83, V::new(-arm.y * omega, arm.x * omega));
        }
        for (i, c) in BUMPERS.iter().enumerate() {
            let d = self.ball - *c;
            let l = d.len();
            if l < 18. && l > 0.001 {
                let n = d * (1. / l);
                self.ball = *c + n * 18.;
                if self.vel.dot(n) < 0. {
                    self.vel = self.vel - n * (2. * self.vel.dot(n)) + n * 105.;
                    self.score += 100 * self.multiplier;
                    self.hits += 1;
                    self.flash[i] = 1.;
                    self.shaking = 0.65;
                    self.lamps[i] = true;
                    self.message = "BUMPER / +100";
                    if self.lamps.iter().all(|x| *x) {
                        self.score += 500 * self.multiplier;
                        self.multiplier = (self.multiplier + 1).min(5);
                        self.lamps = [false; 3];
                        self.message = "BANK COMPLETE / MULTIPLIER UP";
                    }
                }
            }
        }
        // Kickback slings have a cooldown through the shared impact envelope.
        if self.shaking < 0.05
            && self.ball.y > 225.
            && self.ball.y < 266.
            && self.ball.x < 190.
            && (self.ball.x < 72. || self.ball.x > 154.)
        {
            self.vel.x = if self.ball.x < 112. { 140. } else { -140. };
            self.vel.y = -190.;
            self.score += 25 * self.multiplier;
            self.shaking = 0.35;
            self.message = "SLING / +25";
        }
        let speed = self.vel.len();
        if speed > 620. {
            self.vel = self.vel * (620. / speed);
        }
        self.best = self.best.max(self.score);
        if self.ball.y > 373. {
            self.balls = self.balls.saturating_sub(1);
            self.live = false;
            self.ball = V::new(212., 329.);
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
            g.ball = V::new(112., 380.);
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
            ball: V::new(94., 312.),
            vel: V::new(0., 40.),
            ..Game::default()
        };
        for _ in 0..12 {
            g.step(1. / 240.);
        }
        assert!(g.vel.y < 0., "rising flipper must transfer velocity");
    }
}
