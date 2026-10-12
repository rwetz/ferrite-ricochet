//! Board identity and mechanics are separate from color schemes. Only Ferrite is authored today.
use crate::physics::V;
use std::sync::LazyLock;
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

pub struct Board {
    pub key: &'static str,
    pub title: &'static str,
    pub scoop: V,
    pub gate: (V, V),
    pub bumpers: &'static [V; 3],
    pub walls: &'static [(V, V)],
    pub orbit: &'static LazyLock<Rail>,
}
pub const FERRITE: Board = Board {
    key: "ferrite",
    title: "FERROUS",
    scoop: V::new(112., 198.),
    gate: (V::new(28., 219.), V::new(47., 219.)),
    bumpers: &BUMPERS,
    walls: &WALLS,
    orbit: &ORBIT,
};
pub fn for_scheme(_scheme: &str) -> &'static Board {
    &FERRITE
}
pub struct Rail {
    pub points: Vec<V>,
    pub distances: Vec<f32>,
    pub length: f32,
}
impl Rail {
    fn orbit() -> Self {
        let curves = [
            [
                V::new(38., 220.),
                V::new(27., 135.),
                V::new(28., 40.),
                V::new(77., 33.),
            ],
            [
                V::new(77., 33.),
                V::new(125., 16.),
                V::new(190., 28.),
                V::new(192., 83.),
            ],
            [
                V::new(192., 83.),
                V::new(203., 133.),
                V::new(188., 180.),
                V::new(161., 195.),
            ],
        ];
        let mut points = Vec::with_capacity(73);
        for (i, p) in curves.iter().enumerate() {
            for step in 0..=24 {
                if i > 0 && step == 0 {
                    continue;
                }
                let t = step as f32 / 24.;
                let u = 1. - t;
                points.push(
                    p[0] * (u * u * u)
                        + p[1] * (3. * u * u * t)
                        + p[2] * (3. * u * t * t)
                        + p[3] * (t * t * t),
                );
            }
        }
        let mut distances = vec![0.];
        for i in 1..points.len() {
            distances.push(distances[i - 1] + (points[i] - points[i - 1]).len());
        }
        let length = *distances.last().unwrap();
        Self {
            points,
            distances,
            length,
        }
    }
    pub fn sample(&self, d: f32) -> (V, V) {
        let d = d.clamp(0., self.length);
        let i = self
            .distances
            .partition_point(|v| *v <= d)
            .saturating_sub(1)
            .min(self.points.len() - 2);
        let segment = self.points[i + 1] - self.points[i];
        let l = segment.len();
        let t = (d - self.distances[i]) / l;
        (self.points[i] + segment * t, segment * (1. / l))
    }
    pub fn height(&self, d: f32) -> f32 {
        (d / self.length * std::f32::consts::PI).sin().max(0.) * 12.
    }
}
pub static ORBIT: LazyLock<Rail> = LazyLock::new(Rail::orbit);
pub fn mission(stage: u8) -> (&'static str, &'static str) {
    match stage {
        0 => (
            "01 / BREAK THE BANK",
            "Light all three bumper cores to release the orbit gate.",
        ),
        1 => (
            "02 / COMPLETE THE ORBIT",
            "Gate open. Shoot the left rail hard enough to carry the full loop.",
        ),
        2 => (
            "03 / FEED THE REACTOR",
            "Orbit complete. Shoot the center scoop to start overdrive.",
        ),
        _ => (
            "04 / OVERDRIVE",
            "All scoring doubles until the reactor winds down. Keep it alive.",
        ),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rail_is_continuous_and_returns() {
        assert!(ORBIT.length > 350.);
        for pair in ORBIT.points.windows(2) {
            assert!((pair[1] - pair[0]).len() < 20.);
        }
        let (p, t) = ORBIT.sample(ORBIT.length);
        assert!((p - V::new(161., 195.)).len() < 0.1);
        assert!(t.y > 0.);
    }
}
