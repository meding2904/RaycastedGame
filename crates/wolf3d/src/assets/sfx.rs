//! Procedurally synthesised sound effects and a looping music bed.

use super::Rng;

pub const SAMPLE_RATE: u32 = 22050;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sfx {
    KnifeSwing,
    KnifeHit,
    PistolShot,
    MachineGunShot,
    ChaingunShot,
    GunEmpty,
    DoorOpen,
    DoorClose,
    DoorLocked,
    Pushwall,
    GuardSight,
    GuardDeath,
    DogBark,
    DogAttack,
    BossSight,
    PlayerPain,
    PlayerDeath,
    PlayerFootstep,
    PickupHealth,
    PickupAmmo,
    PickupTreasure,
    PickupKey,
    ExtraLife,
    ElevatorSwitch,
}

impl Sfx {
    pub const ALL: [Sfx; 24] = [
        Sfx::KnifeSwing,
        Sfx::KnifeHit,
        Sfx::PistolShot,
        Sfx::MachineGunShot,
        Sfx::ChaingunShot,
        Sfx::GunEmpty,
        Sfx::DoorOpen,
        Sfx::DoorClose,
        Sfx::DoorLocked,
        Sfx::Pushwall,
        Sfx::GuardSight,
        Sfx::GuardDeath,
        Sfx::DogBark,
        Sfx::DogAttack,
        Sfx::BossSight,
        Sfx::PlayerPain,
        Sfx::PlayerDeath,
        Sfx::PlayerFootstep,
        Sfx::PickupHealth,
        Sfx::PickupAmmo,
        Sfx::PickupTreasure,
        Sfx::PickupKey,
        Sfx::ExtraLife,
        Sfx::ElevatorSwitch,
    ];

    pub fn index(self) -> usize {
        Sfx::ALL.iter().position(|s| *s == self).unwrap_or(0)
    }
}

/// The generated sound bank: one mono sample buffer per effect plus the music loop
pub struct Sounds {
    effects: Vec<(Sfx, Vec<f32>)>,
    pub music: Vec<f32>,
}

impl Sounds {
    pub fn generate() -> Self {
        let effects = Sfx::ALL
            .iter()
            .map(|sfx| (*sfx, render(*sfx)))
            .collect::<Vec<_>>();
        let music = build_music();

        Sounds { effects, music }
    }

    pub fn samples(&self, sfx: Sfx) -> &[f32] {
        self.effects
            .iter()
            .find(|(s, _)| *s == sfx)
            .map(|(_, data)| data.as_slice())
            .unwrap_or(&[])
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Wave {
    Sine,
    Square,
    Saw,
    Triangle,
}

/// A mono sample buffer with the handful of primitives every effect is built from
struct Synth {
    data: Vec<f32>,
    rng: Rng,
}

impl Synth {
    fn new(seconds: f32) -> Self {
        let len = (seconds * SAMPLE_RATE as f32) as usize;
        Self {
            data: vec![0.0; len.max(1)],
            rng: Rng::new(0x51FC_0000 + (seconds * 1000.0) as u64),
        }
    }

    fn at(&mut self, index: usize, value: f32) {
        if let Some(slot) = self.data.get_mut(index) {
            *slot = (*slot + value).clamp(-1.0, 1.0);
        }
    }

    fn samples(&self, seconds: f32) -> usize {
        (seconds * SAMPLE_RATE as f32) as usize
    }

    /// White noise with an exponential decay, optionally low pass filtered
    fn noise_burst(&mut self, start: f32, duration: f32, amp: f32, decay: f32, lowpass: f32) {
        let s = self.samples(start);
        let n = self.samples(duration);
        let mut last = 0.0f32;

        for i in 0..n {
            let t = i as f32 / n as f32;
            let env = (1.0 - t).powf(decay);
            let white = self.rng.range_f32(-1.0, 1.0);
            let value = white * (1.0 - lowpass) + last * lowpass;
            last = value;
            self.at(s + i, value * env * amp);
        }
    }

    /// A tone from `f0` to `f1` with an exponential decay
    fn tone(
        &mut self,
        start: f32,
        duration: f32,
        f0: f32,
        f1: f32,
        amp: f32,
        wave: Wave,
        decay: f32,
    ) {
        let s = self.samples(start);
        let n = self.samples(duration);
        let mut phase = 0.0f32;

        for i in 0..n {
            let t = i as f32 / n as f32;
            let freq = f0 + (f1 - f0) * t;
            phase = (phase + freq / SAMPLE_RATE as f32) % 1.0;
            let value = match wave {
                Wave::Sine => (phase * std::f32::consts::TAU).sin(),
                Wave::Square => {
                    if phase < 0.5 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                Wave::Saw => phase * 2.0 - 1.0,
                Wave::Triangle => 1.0 - 4.0 * (phase - 0.5).abs(),
            };
            let env = (1.0 - t).powf(decay);
            self.at(s + i, value * env * amp);
        }
    }

    /// A short click, used to give shots their attack transient
    fn click(&mut self, start: f32, amp: f32) {
        self.noise_burst(start, 0.012, amp, 2.0, 0.0);
    }

    /// A sequence of notes, used for pickups and jingles
    fn arpeggio(&mut self, start: f32, notes: &[f32], step: f32, amp: f32, wave: Wave) {
        for (i, midi) in notes.iter().enumerate() {
            let at = start + i as f32 * step;
            self.tone(at, step * 2.0, freq(*midi), freq(*midi), amp, wave, 2.5);
        }
    }
}

fn freq(midi: f32) -> f32 {
    440.0 * 2f32.powf((midi - 69.0) / 12.0)
}

fn render(sfx: Sfx) -> Vec<f32> {
    let mut s = match sfx {
        Sfx::KnifeSwing => Synth::new(0.30),
        Sfx::KnifeHit => Synth::new(0.30),
        Sfx::PistolShot => Synth::new(0.35),
        Sfx::MachineGunShot => Synth::new(0.30),
        Sfx::ChaingunShot => Synth::new(0.28),
        Sfx::GunEmpty => Synth::new(0.12),
        Sfx::DoorOpen => Synth::new(1.20),
        Sfx::DoorClose => Synth::new(1.00),
        Sfx::DoorLocked => Synth::new(0.40),
        Sfx::Pushwall => Synth::new(1.80),
        Sfx::GuardSight => Synth::new(0.70),
        Sfx::GuardDeath => Synth::new(0.90),
        Sfx::DogBark => Synth::new(0.45),
        Sfx::DogAttack => Synth::new(0.60),
        Sfx::BossSight => Synth::new(0.90),
        Sfx::PlayerPain => Synth::new(0.40),
        Sfx::PlayerDeath => Synth::new(1.60),
        Sfx::PlayerFootstep => Synth::new(0.14),
        Sfx::PickupHealth => Synth::new(0.50),
        Sfx::PickupAmmo => Synth::new(0.40),
        Sfx::PickupTreasure => Synth::new(0.60),
        Sfx::PickupKey => Synth::new(0.60),
        Sfx::ExtraLife => Synth::new(1.10),
        Sfx::ElevatorSwitch => Synth::new(1.30),
    };

    match sfx {
        Sfx::KnifeSwing => {
            s.noise_burst(0.0, 0.26, 0.35, 3.0, 0.55);
            s.tone(0.02, 0.20, 900.0, 300.0, 0.10, Wave::Triangle, 3.0);
        }
        Sfx::KnifeHit => {
            s.noise_burst(0.0, 0.10, 0.55, 6.0, 0.75);
            s.tone(0.0, 0.14, 180.0, 70.0, 0.45, Wave::Sine, 5.0);
        }
        Sfx::PistolShot => {
            s.click(0.0, 0.5);
            s.noise_burst(0.0, 0.22, 0.75, 7.0, 0.25);
            s.tone(0.0, 0.20, 260.0, 60.0, 0.5, Wave::Sine, 6.0);
        }
        Sfx::MachineGunShot => {
            s.click(0.0, 0.45);
            s.noise_burst(0.0, 0.18, 0.65, 8.0, 0.35);
            s.tone(0.0, 0.16, 220.0, 70.0, 0.4, Wave::Sine, 7.0);
        }
        Sfx::ChaingunShot => {
            s.click(0.0, 0.4);
            s.noise_burst(0.0, 0.16, 0.6, 9.0, 0.45);
            s.tone(0.0, 0.14, 190.0, 80.0, 0.35, Wave::Sine, 8.0);
        }
        Sfx::GunEmpty => {
            s.click(0.0, 0.35);
            s.tone(0.0, 0.05, 1400.0, 800.0, 0.2, Wave::Square, 4.0);
        }
        Sfx::DoorOpen => {
            s.noise_burst(0.0, 1.05, 0.22, 0.8, 0.85);
            s.tone(0.0, 1.0, 120.0, 190.0, 0.22, Wave::Saw, 1.2);
            s.tone(0.1, 0.8, 62.0, 74.0, 0.18, Wave::Triangle, 1.0);
        }
        Sfx::DoorClose => {
            s.noise_burst(0.0, 0.85, 0.26, 1.0, 0.85);
            s.tone(0.0, 0.8, 170.0, 90.0, 0.22, Wave::Saw, 1.4);
            s.noise_burst(0.8, 0.12, 0.5, 5.0, 0.5);
        }
        Sfx::DoorLocked => {
            s.tone(0.0, 0.12, 300.0, 260.0, 0.35, Wave::Square, 5.0);
            s.tone(0.14, 0.20, 220.0, 180.0, 0.3, Wave::Square, 4.0);
        }
        Sfx::Pushwall => {
            s.noise_burst(0.0, 1.7, 0.30, 0.6, 0.9);
            s.tone(0.0, 1.6, 46.0, 58.0, 0.30, Wave::Triangle, 0.8);
            for i in 0..7 {
                s.noise_burst(0.1 + i as f32 * 0.22, 0.16, 0.18, 4.0, 0.6);
            }
        }
        Sfx::GuardSight => {
            s.tone(0.0, 0.22, 340.0, 210.0, 0.42, Wave::Saw, 1.5);
            s.noise_burst(0.0, 0.22, 0.18, 2.0, 0.6);
            s.tone(0.24, 0.36, 300.0, 150.0, 0.38, Wave::Saw, 1.6);
            s.noise_burst(0.24, 0.38, 0.16, 2.0, 0.6);
        }
        Sfx::BossSight => {
            s.tone(0.0, 0.45, 190.0, 110.0, 0.45, Wave::Saw, 1.0);
            s.tone(0.35, 0.5, 150.0, 80.0, 0.4, Wave::Saw, 1.2);
            s.noise_burst(0.0, 0.8, 0.2, 1.2, 0.7);
        }
        Sfx::GuardDeath => {
            s.tone(0.0, 0.75, 260.0, 70.0, 0.4, Wave::Saw, 1.4);
            s.noise_burst(0.0, 0.8, 0.28, 1.5, 0.55);
        }
        Sfx::DogBark => {
            s.tone(0.0, 0.12, 420.0, 240.0, 0.45, Wave::Saw, 4.0);
            s.noise_burst(0.0, 0.12, 0.3, 5.0, 0.5);
            s.tone(0.18, 0.14, 380.0, 200.0, 0.4, Wave::Saw, 4.0);
        }
        Sfx::DogAttack => {
            for i in 0..3 {
                let at = i as f32 * 0.16;
                s.tone(at, 0.14, 520.0, 260.0, 0.42, Wave::Saw, 3.5);
                s.noise_burst(at, 0.14, 0.34, 4.0, 0.35);
            }
        }
        Sfx::PlayerPain => {
            s.tone(0.0, 0.28, 300.0, 180.0, 0.40, Wave::Saw, 2.0);
            s.noise_burst(0.0, 0.26, 0.16, 3.0, 0.5);
        }
        Sfx::PlayerDeath => {
            s.tone(0.0, 1.4, 300.0, 60.0, 0.42, Wave::Saw, 1.0);
            s.tone(0.1, 1.2, 220.0, 50.0, 0.3, Wave::Triangle, 1.0);
            s.noise_burst(0.5, 1.0, 0.12, 1.5, 0.8);
        }
        Sfx::PlayerFootstep => {
            s.noise_burst(0.0, 0.10, 0.18, 5.0, 0.85);
        }
        Sfx::PickupHealth => {
            s.arpeggio(0.0, &[69.0, 76.0], 0.10, 0.32, Wave::Square);
        }
        Sfx::PickupAmmo => {
            s.arpeggio(0.0, &[64.0, 71.0], 0.08, 0.30, Wave::Square);
        }
        Sfx::PickupTreasure => {
            s.tone(0.0, 0.5, freq(84.0), freq(84.0), 0.28, Wave::Sine, 3.2);
            s.tone(0.0, 0.5, freq(91.0), freq(91.0), 0.18, Wave::Sine, 3.6);
            s.tone(0.06, 0.4, freq(88.0), freq(88.0), 0.16, Wave::Sine, 3.0);
        }
        Sfx::PickupKey => {
            s.tone(0.0, 0.35, 1200.0, 900.0, 0.22, Wave::Sine, 3.0);
            s.tone(0.02, 0.3, 1800.0, 1400.0, 0.14, Wave::Sine, 3.5);
            s.tone(0.3, 0.25, 900.0, 700.0, 0.18, Wave::Sine, 3.0);
        }
        Sfx::ExtraLife => {
            s.arpeggio(0.0, &[60.0, 64.0, 67.0, 72.0], 0.11, 0.30, Wave::Square);
            s.tone(0.44, 0.6, freq(76.0), freq(76.0), 0.26, Wave::Square, 1.6);
        }
        Sfx::ElevatorSwitch => {
            s.click(0.0, 0.4);
            s.tone(0.0, 0.25, 600.0, 400.0, 0.3, Wave::Square, 3.0);
            s.tone(0.3, 0.9, 180.0, 300.0, 0.24, Wave::Saw, 1.2);
            s.noise_burst(0.3, 0.9, 0.18, 1.0, 0.85);
        }
    }

    s.data
}

/// A 16 second chiptune loop: minor key march, square lead, bass on the beat, noise percussion.
fn build_music() -> Vec<f32> {
    const BPM: f32 = 132.0;
    const BEAT: f32 = 60.0 / BPM;

    let bars = 8usize;
    let total = bars as f32 * 4.0 * BEAT;
    let mut s = Synth::new(total);

    let roots = [57.0, 53.0, 60.0, 55.0];
    let lead: [[f32; 8]; 4] = [
        [69.0, 72.0, 76.0, 74.0, 72.0, 69.0, 67.0, 69.0],
        [65.0, 69.0, 72.0, 71.0, 69.0, 65.0, 64.0, 65.0],
        [72.0, 74.0, 76.0, 79.0, 76.0, 74.0, 72.0, 71.0],
        [67.0, 71.0, 74.0, 72.0, 71.0, 67.0, 65.0, 67.0],
    ];

    for bar in 0..bars {
        let bar_at = bar as f32 * 4.0 * BEAT;
        let section = bar % 4;
        let root = roots[section];

        for beat in 0..4 {
            let at = bar_at + beat as f32 * BEAT;
            let note = if beat % 2 == 0 {
                root - 12.0
            } else {
                root - 7.0
            };
            s.tone(
                at,
                BEAT * 0.9,
                freq(note),
                freq(note),
                0.24,
                Wave::Square,
                0.6,
            );
        }

        let transpose = root - 69.0;
        for (i, note) in lead[section].iter().enumerate() {
            let at = bar_at + i as f32 * (BEAT / 2.0);
            let midi = note + transpose;
            let dur = if i % 4 == 3 { BEAT * 0.8 } else { BEAT * 0.45 };
            s.tone(at, dur, freq(midi), freq(midi), 0.16, Wave::Square, 0.7);
            if i % 2 == 1 {
                s.tone(
                    at,
                    dur * 0.6,
                    freq(midi + 12.0),
                    freq(midi + 12.0),
                    0.05,
                    Wave::Triangle,
                    1.2,
                );
            }
        }

        for beat in 0..8 {
            let at = bar_at + beat as f32 * (BEAT / 2.0);
            match beat % 4 {
                0 => {
                    s.tone(at, 0.11, 90.0, 42.0, 0.32, Wave::Sine, 4.0);
                }
                2 => {
                    s.noise_burst(at, 0.10, 0.20, 6.0, 0.72);
                }
                _ => {
                    s.noise_burst(at, 0.045, 0.09, 8.0, 0.2);
                }
            }
        }
    }

    let seam = s.samples(0.05);
    let len = s.data.len();
    for i in 0..seam.min(len / 2) {
        let f = i as f32 / seam as f32;
        s.data[i] *= f;
        s.data[len - 1 - i] *= f;
    }

    s.data
}
