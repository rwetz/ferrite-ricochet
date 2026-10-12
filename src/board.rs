//! Every Ferrite scheme owns its collision layout, routes, target sequence and etched playfield.
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

#[derive(Clone, Copy)]
pub struct DropTarget {
    pub center: V,
    pub normal: V,
}
pub struct Board {
    pub key: &'static str,
    pub title: &'static str,
    pub design: u8,
    pub scoop: V,
    pub gate: (V, V),
    pub ramp_gate: (V, V),
    pub bumpers: &'static [V; 3],
    pub walls: &'static [(V, V)],
    pub orbit: &'static LazyLock<Rail>,
    pub ramp: &'static LazyLock<Rail>,
    pub targets: [DropTarget; 3],
    pub order: [usize; 3],
    pub lanes: [V; 3],
}
const fn gate(entry: V) -> (V, V) {
    (V::new(entry.x - 9., entry.y), V::new(entry.x + 9., entry.y))
}
const fn target_bank(points: [V; 3]) -> [DropTarget; 3] {
    [
        DropTarget {
            center: points[0],
            normal: V::new(0., 1.),
        },
        DropTarget {
            center: points[1],
            normal: V::new(0., 1.),
        },
        DropTarget {
            center: points[2],
            normal: V::new(0., 1.),
        },
    ]
}
const fn guides(extra: [(V, V); 2]) -> [(V, V); 14] {
    let mut walls = [(V::new(0., 0.), V::new(0., 0.)); 14];
    let mut i = 0;
    while i < 12 {
        walls[i] = WALLS[i];
        i += 1;
    }
    walls[12] = extra[0];
    walls[13] = extra[1];
    walls
}
macro_rules! v {
    ($x:expr,$y:expr) => {
        V::new($x, $y)
    };
}
macro_rules! map {
    ($name:ident,$orbit:ident,$ramp:ident,$walls:ident,$key:literal,$title:literal,$design:expr,$bumpers:expr,$scoop:expr,$targets:expr,$order:expr,$lanes:expr,$orbit_curves:expr,$ramp_curves:expr,$extra:expr) => {
        static $orbit: LazyLock<Rail> = LazyLock::new(|| Rail::curves($orbit_curves, 12., 24));
        static $ramp: LazyLock<Rail> = LazyLock::new(|| Rail::curves($ramp_curves, 32., 16));
        static $walls: [(V, V); 14] = guides($extra);
        pub static $name: Board = Board {
            key: $key,
            title: $title,
            design: $design,
            bumpers: &$bumpers,
            scoop: $scoop,
            targets: target_bank($targets),
            order: $order,
            lanes: $lanes,
            gate: gate($orbit_curves[0][0]),
            ramp_gate: gate($ramp_curves[0][0]),
            walls: &$walls,
            orbit: &$orbit,
            ramp: &$ramp,
        };
    };
}

map!(
    FERRITE,
    ORBIT,
    FERRITE_RAMP,
    FERRITE_WALLS,
    "ferrite",
    "FERROUS",
    0,
    [v!(78., 112.), v!(150., 96.), v!(120., 161.)],
    v!(112., 198.),
    [v!(83., 244.), v!(107., 241.), v!(131., 238.)],
    [0, 1, 2],
    [v!(65., 54.), v!(107., 54.), v!(149., 54.)],
    [
        [v!(38., 220.), v!(27., 135.), v!(28., 40.), v!(77., 33.)],
        [v!(77., 33.), v!(125., 16.), v!(190., 28.), v!(192., 83.)],
        [
            v!(192., 83.),
            v!(203., 133.),
            v!(188., 180.),
            v!(161., 195.)
        ]
    ],
    [
        [
            v!(174., 247.),
            v!(194., 207.),
            v!(190., 136.),
            v!(164., 103.)
        ],
        [v!(164., 103.), v!(140., 74.), v!(104., 64.), v!(80., 92.)],
        [v!(80., 92.), v!(57., 122.), v!(46., 180.), v!(66., 208.)]
    ],
    [
        (v!(37., 175.), v!(52., 183.)),
        (v!(188., 174.), v!(176., 188.))
    ]
);

map!(
    MONO,
    MONO_ORBIT,
    MONO_RAMP,
    MONO_WALLS,
    "mono",
    "MONOLITH",
    1,
    [v!(75., 112.), v!(151., 112.), v!(113., 160.)],
    v!(113., 206.),
    [v!(74., 242.), v!(113., 234.), v!(152., 242.)],
    [0, 2, 1],
    [v!(61., 54.), v!(113., 54.), v!(165., 54.)],
    [
        [v!(181., 224.), v!(193., 143.), v!(198., 46.), v!(151., 33.)],
        [v!(151., 33.), v!(109., 18.), v!(44., 23.), v!(33., 87.)],
        [v!(33., 87.), v!(26., 153.), v!(43., 198.), v!(73., 215.)]
    ],
    [
        [v!(45., 247.), v!(30., 200.), v!(45., 133.), v!(77., 112.)],
        [v!(77., 112.), v!(111., 87.), v!(155., 77.), v!(178., 108.)],
        [
            v!(178., 108.),
            v!(193., 145.),
            v!(170., 193.),
            v!(151., 218.)
        ]
    ],
    [
        (v!(34., 142.), v!(56., 152.)),
        (v!(184., 142.), v!(162., 152.))
    ]
);

map!(
    GRAPHITE,
    GRAPHITE_ORBIT,
    GRAPHITE_RAMP,
    GRAPHITE_WALLS,
    "graphite",
    "FOUNDRY",
    2,
    [v!(65., 102.), v!(120., 136.), v!(164., 106.)],
    v!(95., 196.),
    [v!(79., 228.), v!(106., 245.), v!(140., 253.)],
    [1, 0, 2],
    [v!(63., 56.), v!(108., 52.), v!(153., 56.)],
    [
        [v!(37., 211.), v!(27., 152.), v!(35., 64.), v!(63., 43.)],
        [v!(63., 43.), v!(111., 24.), v!(170., 28.), v!(188., 73.)],
        [
            v!(188., 73.),
            v!(202., 144.),
            v!(169., 182.),
            v!(146., 200.)
        ]
    ],
    [
        [
            v!(174., 250.),
            v!(182., 199.),
            v!(144., 180.),
            v!(124., 161.)
        ],
        [v!(124., 161.), v!(100., 136.), v!(76., 90.), v!(60., 99.)],
        [v!(60., 99.), v!(38., 121.), v!(49., 180.), v!(71., 207.)]
    ],
    [
        (v!(44., 178.), v!(62., 191.)),
        (v!(186., 181.), v!(166., 192.))
    ]
);

map!(
    SLATE,
    SLATE_ORBIT,
    SLATE_RAMP,
    SLATE_WALLS,
    "slate",
    "BREAKWATER",
    3,
    [v!(64., 106.), v!(112., 118.), v!(160., 106.)],
    v!(112., 195.),
    [v!(88., 248.), v!(113., 248.), v!(138., 248.)],
    [1, 0, 2],
    [v!(59., 59.), v!(112., 52.), v!(165., 59.)],
    [
        [v!(41., 228.), v!(26., 173.), v!(49., 152.), v!(41., 104.)],
        [v!(41., 104.), v!(27., 50.), v!(66., 22.), v!(118., 36.)],
        [v!(118., 36.), v!(188., 32.), v!(202., 123.), v!(172., 191.)]
    ],
    [
        [
            v!(176., 248.),
            v!(196., 204.),
            v!(183., 157.),
            v!(156., 147.)
        ],
        [v!(156., 147.), v!(120., 133.), v!(110., 65.), v!(77., 78.)],
        [v!(77., 78.), v!(39., 104.), v!(52., 160.), v!(79., 191.)]
    ],
    [
        (v!(42., 153.), v!(59., 166.)),
        (v!(187., 153.), v!(170., 166.))
    ]
);

map!(
    CONCRETE,
    CONCRETE_ORBIT,
    CONCRETE_RAMP,
    CONCRETE_WALLS,
    "concrete",
    "SWITCHYARD",
    4,
    [v!(67., 129.), v!(115., 93.), v!(160., 142.)],
    v!(128., 203.),
    [v!(83., 224.), v!(104., 246.), v!(147., 233.)],
    [1, 2, 0],
    [v!(62., 57.), v!(109., 51.), v!(159., 57.)],
    [
        [
            v!(180., 237.),
            v!(199., 192.),
            v!(195., 115.),
            v!(177., 95.)
        ],
        [v!(177., 95.), v!(174., 55.), v!(174., 30.), v!(120., 31.)],
        [v!(120., 31.), v!(47., 22.), v!(23., 99.), v!(61., 191.)]
    ],
    [
        [v!(44., 247.), v!(30., 210.), v!(63., 187.), v!(69., 166.)],
        [v!(69., 166.), v!(75., 126.), v!(91., 58.), v!(137., 68.)],
        [v!(137., 68.), v!(192., 83.), v!(169., 165.), v!(151., 192.)]
    ],
    [
        (v!(57., 184.), v!(82., 205.)),
        (v!(164., 182.), v!(145., 203.))
    ]
);

map!(
    HARBOR,
    HARBOR_ORBIT,
    HARBOR_RAMP,
    HARBOR_WALLS,
    "harbor",
    "DRYDOCK",
    5,
    [v!(95., 102.), v!(151., 137.), v!(67., 155.)],
    v!(125., 199.),
    [v!(78., 242.), v!(122., 250.), v!(157., 231.)],
    [2, 0, 1],
    [v!(64., 55.), v!(111., 52.), v!(158., 55.)],
    [
        [v!(38., 225.), v!(24., 170.), v!(28., 80.), v!(52., 63.)],
        [v!(52., 63.), v!(61., 20.), v!(128., 19.), v!(173., 47.)],
        [v!(173., 47.), v!(207., 84.), v!(189., 153.), v!(171., 186.)]
    ],
    [
        [
            v!(174., 248.),
            v!(183., 205.),
            v!(157., 186.),
            v!(151., 162.)
        ],
        [v!(151., 162.), v!(148., 126.), v!(113., 50.), v!(76., 71.)],
        [v!(76., 71.), v!(39., 100.), v!(50., 170.), v!(83., 203.)]
    ],
    [
        (v!(36., 177.), v!(54., 190.)),
        (v!(181., 177.), v!(165., 194.))
    ]
);

map!(
    CYANOTYPE,
    CYANOTYPE_ORBIT,
    CYANOTYPE_RAMP,
    CYANOTYPE_WALLS,
    "cyanotype",
    "BLUEPRINT",
    6,
    [v!(79., 100.), v!(147., 101.), v!(111., 145.)],
    v!(111., 211.),
    [v!(87., 234.), v!(111., 249.), v!(143., 234.)],
    [0, 2, 1],
    [v!(65., 52.), v!(111., 56.), v!(157., 52.)],
    [
        [
            v!(180., 225.),
            v!(197., 159.),
            v!(191., 114.),
            v!(177., 86.)
        ],
        [v!(177., 86.), v!(164., 19.), v!(96., 22.), v!(60., 45.)],
        [v!(60., 45.), v!(21., 98.), v!(33., 166.), v!(63., 192.)]
    ],
    [
        [v!(45., 247.), v!(29., 204.), v!(51., 170.), v!(72., 151.)],
        [v!(72., 151.), v!(102., 124.), v!(153., 65.), v!(175., 93.)],
        [
            v!(175., 93.),
            v!(192., 130.),
            v!(167., 167.),
            v!(146., 193.)
        ]
    ],
    [
        (v!(35., 159.), v!(57., 169.)),
        (v!(190., 160.), v!(168., 169.))
    ]
);

map!(
    PHOSPHOR,
    PHOSPHOR_ORBIT,
    PHOSPHOR_RAMP,
    PHOSPHOR_WALLS,
    "phosphor",
    "CRITICAL",
    7,
    [v!(111., 93.), v!(140., 137.), v!(89., 153.)],
    v!(127., 202.),
    [v!(76., 248.), v!(107., 238.), v!(142., 250.)],
    [1, 0, 2],
    [v!(61., 55.), v!(111., 48.), v!(164., 55.)],
    [
        [v!(38., 227.), v!(31., 174.), v!(48., 129.), v!(41., 91.)],
        [v!(41., 91.), v!(28., 31.), v!(117., 17.), v!(165., 48.)],
        [v!(165., 48.), v!(204., 84.), v!(189., 158.), v!(164., 188.)]
    ],
    [
        [
            v!(175., 246.),
            v!(199., 203.),
            v!(162., 186.),
            v!(145., 165.)
        ],
        [v!(145., 165.), v!(115., 126.), v!(73., 69.), v!(61., 105.)],
        [v!(61., 105.), v!(43., 153.), v!(85., 173.), v!(89., 202.)]
    ],
    [
        (v!(47., 197.), v!(64., 208.)),
        (v!(188., 171.), v!(171., 191.))
    ]
);

map!(
    VERDIGRIS,
    VERDIGRIS_ORBIT,
    VERDIGRIS_RAMP,
    VERDIGRIS_WALLS,
    "verdigris",
    "OXIDE",
    8,
    [v!(68., 105.), v!(102., 151.), v!(154., 119.)],
    v!(93., 199.),
    [v!(87., 239.), v!(124., 242.), v!(154., 220.)],
    [2, 1, 0],
    [v!(65., 54.), v!(112., 49.), v!(160., 60.)],
    [
        [v!(39., 231.), v!(22., 179.), v!(45., 138.), v!(34., 99.)],
        [v!(34., 99.), v!(27., 43.), v!(74., 23.), v!(127., 35.)],
        [v!(127., 35.), v!(196., 45.), v!(187., 132.), v!(164., 179.)]
    ],
    [
        [
            v!(176., 247.),
            v!(190., 211.),
            v!(150., 186.),
            v!(140., 161.)
        ],
        [v!(140., 161.), v!(135., 117.), v!(87., 70.), v!(61., 94.)],
        [v!(61., 94.), v!(45., 122.), v!(59., 173.), v!(78., 203.)]
    ],
    [
        (v!(37., 164.), v!(53., 185.)),
        (v!(185., 185.), v!(163., 199.))
    ]
);

map!(
    BRUISE,
    BRUISE_ORBIT,
    BRUISE_RAMP,
    BRUISE_WALLS,
    "bruise",
    "BLACKOUT",
    9,
    [v!(98., 110.), v!(158., 151.), v!(73., 167.)],
    v!(127., 209.),
    [v!(86., 240.), v!(122., 252.), v!(147., 231.)],
    [1, 2, 0],
    [v!(63., 52.), v!(109., 61.), v!(161., 51.)],
    [
        [v!(39., 228.), v!(28., 174.), v!(57., 139.), v!(47., 105.)],
        [v!(47., 105.), v!(30., 50.), v!(108., 20.), v!(151., 43.)],
        [v!(151., 43.), v!(198., 81.), v!(173., 112.), v!(179., 184.)]
    ],
    [
        [
            v!(175., 247.),
            v!(184., 198.),
            v!(149., 172.),
            v!(137., 150.)
        ],
        [v!(137., 150.), v!(110., 101.), v!(86., 57.), v!(58., 90.)],
        [v!(58., 90.), v!(29., 121.), v!(55., 177.), v!(85., 204.)]
    ],
    [
        (v!(35., 192.), v!(56., 211.)),
        (v!(187., 196.), v!(166., 211.))
    ]
);

pub static ALL: [&Board; 10] = [
    &FERRITE, &MONO, &GRAPHITE, &SLATE, &CONCRETE, &HARBOR, &CYANOTYPE, &PHOSPHOR, &VERDIGRIS,
    &BRUISE,
];
pub fn for_scheme(key: &str) -> &'static Board {
    ALL.iter()
        .copied()
        .find(|b| b.key == key)
        .unwrap_or(&FERRITE)
}
pub struct Rail {
    pub points: Vec<V>,
    pub distances: Vec<f32>,
    pub length: f32,
    pub height_peak: f32,
}
impl Rail {
    fn curves(curves: [[V; 4]; 3], height_peak: f32, steps: usize) -> Self {
        let mut points = Vec::with_capacity(73);
        for (i, p) in curves.iter().enumerate() {
            for step in 0..=steps {
                if i > 0 && step == 0 {
                    continue;
                }
                let t = step as f32 / steps as f32;
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
            height_peak,
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
        (d / self.length * std::f32::consts::PI).sin().max(0.) * self.height_peak
    }
    pub fn slope(&self, d: f32) -> f32 {
        self.height_peak * std::f32::consts::PI / self.length
            * (d / self.length * std::f32::consts::PI).cos()
    }
}
pub fn mission(board: &Board, stage: u8) -> (&'static str, String) {
    match stage {
        0 => (
            "01 / BREAK THE BANK",
            "Light all three bumper cores to release the orbit gate.".into(),
        ),
        1 => (
            "02 / COMPLETE THE ORBIT",
            format!(
                "Gate open. Shoot the {} orbit hard enough to carry the loop.",
                if board.orbit.points[0].x < 122. {
                    "left"
                } else {
                    "right"
                }
            ),
        ),
        2 => (
            "03 / FEED THE REACTOR",
            "Orbit complete. Shoot the reactor scoop to start overdrive.".into(),
        ),
        _ => (
            "04 / OVERDRIVE",
            "All scoring doubles until the reactor winds down. Keep it alive.".into(),
        ),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_scheme_has_a_unique_real_layout() {
        for (i, a) in ALL.iter().enumerate() {
            assert!(std::ptr::eq(for_scheme(a.key), *a));
            for b in &ALL[i + 1..] {
                assert_ne!(a.key, b.key);
                assert_ne!(a.bumpers, b.bumpers);
                assert_ne!(a.orbit.points, b.orbit.points);
                assert_ne!(a.ramp.points, b.ramp.points);
            }
        }
    }
    #[test]
    fn authored_routes_are_finite_continuous_and_inside_cabinet() {
        for b in ALL {
            for rail in [b.orbit, b.ramp] {
                assert!(rail.length > 250.);
                for pair in rail.points.windows(2) {
                    assert!((pair[1] - pair[0]).len() > 0.01 && (pair[1] - pair[0]).len() < 25.);
                }
                for p in &rail.points {
                    assert!(p.x >= 20. && p.x <= 205. && p.y >= 20. && p.y <= 260.);
                }
                assert!(rail.height(rail.length).abs() < 0.01);
            }
        }
    }
    #[test]
    fn all_target_orders_are_permutations() {
        for b in ALL {
            let mut order = b.order;
            order.sort();
            assert_eq!(order, [0, 1, 2]);
        }
    }
}
