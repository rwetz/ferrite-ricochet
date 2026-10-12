//! Short synthesized mechanical cues. Windows uses the system wave mixer, with no file I/O.
use std::time::{Duration, Instant};
#[derive(Clone, Copy)]
pub enum Cue {
    Flipper,
    Launch,
    Bumper,
    Sling,
    Bank,
    Drain,
}
pub struct Audio {
    pub enabled: bool,
    last: Instant,
}
impl Default for Audio {
    fn default() -> Self {
        Self {
            enabled: cfg!(windows),
            last: Instant::now() - Duration::from_secs(1),
        }
    }
}
impl Audio {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            ..Self::default()
        }
    }
    pub fn play(&mut self, cue: Cue) {
        if !self.enabled {
            return;
        }
        if matches!(cue, Cue::Flipper) && self.last.elapsed() < Duration::from_millis(35) {
            return;
        }
        self.last = Instant::now();
        #[cfg(windows)]
        system::play(cue);
        #[cfg(not(windows))]
        let _ = cue;
    }
}
#[cfg(windows)]
mod system {
    use super::Cue;
    use std::sync::LazyLock;
    #[link(name = "winmm")]
    unsafe extern "system" {
        fn PlaySoundA(sound: *const u8, module: *mut std::ffi::c_void, flags: u32) -> i32;
    }
    static WAVES: LazyLock<[Vec<u8>; 6]> = LazyLock::new(|| {
        [
            (130., 60., 0.035),
            (230., 550., 0.13),
            (740., 430., 0.095),
            (185., 90., 0.055),
            (880., 1320., 0.18),
            (180., 45., 0.24),
        ]
        .map(|(a, b, t)| wave(a, b, t))
    });
    fn wave(from: f32, to: f32, duration: f32) -> Vec<u8> {
        let rate = 22050u32;
        let n = (duration * rate as f32) as u32;
        let bytes = n * 2;
        let mut out = Vec::with_capacity(44 + bytes as usize);
        out.extend(b"RIFF");
        out.extend((36 + bytes).to_le_bytes());
        out.extend(b"WAVEfmt ");
        out.extend(16u32.to_le_bytes());
        out.extend(1u16.to_le_bytes());
        out.extend(1u16.to_le_bytes());
        out.extend(rate.to_le_bytes());
        out.extend((rate * 2).to_le_bytes());
        out.extend(2u16.to_le_bytes());
        out.extend(16u16.to_le_bytes());
        out.extend(b"data");
        out.extend(bytes.to_le_bytes());
        let mut phase = 0.;
        for i in 0..n {
            let t = i as f32 / n as f32;
            phase += std::f32::consts::TAU * (from + (to - from) * t) / rate as f32;
            let attack = (t * 40.).min(1.);
            let v =
                (phase.sin() * 0.8 + (phase * 2.).sin() * 0.2) * attack * (1. - t).powi(2) * 0.16;
            out.extend(((v * 32767.) as i16).to_le_bytes());
        }
        out
    }
    pub fn play(cue: Cue) {
        let bytes = &WAVES[cue as usize]; // Static backing memory survives asynchronous playback.
        unsafe {
            PlaySoundA(
                bytes.as_ptr(),
                std::ptr::null_mut(),
                0x0001 | 0x0002 | 0x0004,
            );
        }
    }
}
