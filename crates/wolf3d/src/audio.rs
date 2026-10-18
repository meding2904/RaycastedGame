//! Sound: a rodio output stream for the generated effects and the music bed.

use rodio::{buffer::SamplesBuffer, OutputStream, OutputStreamBuilder, Sink, Source};

use crate::assets::sfx::{Sfx, Sounds, SAMPLE_RATE};

pub struct Audio {
    /// The stream has to stay alive for as long as sound should play
    stream: Option<OutputStream>,
    music: Option<Sink>,
    effects: Vec<(Sfx, Vec<f32>)>,
    music_samples: Vec<f32>,
    pub muted: bool,
}

impl Audio {
    pub fn new(sounds: &Sounds) -> Self {
        let effects = Sfx::ALL
            .iter()
            .map(|sfx| (*sfx, sounds.samples(*sfx).to_vec()))
            .collect();
        let music_samples = sounds.music.clone();

        match OutputStreamBuilder::open_default_stream() {
            Ok(stream) => Self {
                stream: Some(stream),
                music: None,
                effects,
                music_samples,
                muted: false,
            },
            Err(err) => {
                eprintln!("no audio device ({err}), running silently");
                Self {
                    stream: None,
                    music: None,
                    effects,
                    music_samples,
                    muted: false,
                }
            }
        }
    }

    /// An audio backend with no device at all, used by the tests and the frame harness
    pub fn silent(sounds: &Sounds) -> Self {
        Self {
            stream: None,
            music: None,
            effects: Sfx::ALL
                .iter()
                .map(|sfx| (*sfx, sounds.samples(*sfx).to_vec()))
                .collect(),
            music_samples: sounds.music.clone(),
            muted: false,
        }
    }

    pub fn is_available(&self) -> bool {
        self.stream.is_some()
    }

    pub fn play(&self, sfx: Sfx) {
        if self.muted {
            return;
        }
        let Some(stream) = &self.stream else { return };
        let Some((_, data)) = self.effects.iter().find(|(name, _)| *name == sfx) else {
            return;
        };
        if data.is_empty() {
            return;
        }

        stream
            .mixer()
            .add(SamplesBuffer::new(1, SAMPLE_RATE, data.clone()));
    }

    /// Starts the music loop, if it is not already running
    pub fn start_music(&mut self) {
        if self.muted || self.music.is_some() || self.music_samples.is_empty() {
            return;
        }

        let sink = {
            let Some(stream) = &self.stream else { return };
            let sink = Sink::connect_new(stream.mixer());
            sink.set_volume(0.3);
            sink.append(
                SamplesBuffer::new(1, SAMPLE_RATE, self.music_samples.clone()).repeat_infinite(),
            );
            sink
        };

        self.music = Some(sink);
    }

    pub fn stop_music(&mut self) {
        if let Some(sink) = self.music.take() {
            sink.stop();
        }
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        if self.muted {
            self.stop_music();
        } else {
            self.start_music();
        }
    }

    /// How many effects are in the bank, for the harness to report on
    pub fn effect_count(&self) -> usize {
        self.effects.len()
    }

    /// The loudest sample in an effect, which is how the harness checks that the synthesis
    pub fn peak(&self, sfx: Sfx) -> f32 {
        self.effects
            .iter()
            .find(|(name, _)| *name == sfx)
            .map(|(_, data)| {
                data.iter()
                    .fold(0.0f32, |peak, sample| peak.max(sample.abs()))
            })
            .unwrap_or(0.0)
    }

    pub fn music_seconds(&self) -> f32 {
        self.music_samples.len() as f32 / SAMPLE_RATE as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_silent_backend_still_holds_every_effect() {
        let sounds = Sounds::generate();
        let audio = Audio::silent(&sounds);
        assert_eq!(audio.effect_count(), Sfx::ALL.len());
        assert!(!audio.is_available());
    }

    #[test]
    fn every_effect_actually_makes_a_sound() {
        let sounds = Sounds::generate();
        let audio = Audio::silent(&sounds);

        for sfx in Sfx::ALL {
            let peak = audio.peak(sfx);
            assert!(peak > 0.05, "{sfx:?} is silent (peak {peak})");
            assert!(peak <= 1.0, "{sfx:?} clips (peak {peak})");
        }
    }

    #[test]
    fn the_music_bed_is_a_few_seconds_long() {
        let sounds = Sounds::generate();
        let audio = Audio::silent(&sounds);
        assert!(
            audio.music_seconds() > 5.0,
            "music is {}s",
            audio.music_seconds()
        );
    }

    #[test]
    fn playing_without_a_device_is_harmless() {
        let sounds = Sounds::generate();
        let mut audio = Audio::silent(&sounds);
        audio.start_music();
        audio.play(Sfx::PistolShot);
        assert!(!audio.muted);
        audio.toggle_mute();
        assert!(audio.muted);
    }
}
