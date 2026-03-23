// Music state machine — manages looping music layers.
// Transitions are handled by stopping the old sink and starting a new one.
// Loops are implemented by repeating the synthesized buffer.

use crate::SoundBank;
use chaos_rpg_core::audio_events::MusicState;
use chaos_rpg_core::audio_synth as synth;
use rodio::{Decoder, OutputStreamHandle, Sink, Source};
use std::io::Cursor;

pub struct MusicSystem {
    current_state: MusicState,
    sink: Option<Sink>,
    floor_seed: u64,
}

impl MusicSystem {
    pub fn new(handle: &OutputStreamHandle, bank: &SoundBank) -> Self {
        let mut sys = Self {
            current_state: MusicState::Silence,
            sink: None,
            floor_seed: 0,
        };
        sys.transition_to(MusicState::MainMenu, handle, bank);
        sys
    }

    pub fn transition_to(&mut self, new_state: MusicState, handle: &OutputStreamHandle, _bank: &SoundBank) {
        if new_state == self.current_state { return; }

        // Stop the old sink (drop = stop)
        self.sink = None;
        self.current_state = new_state;

        let wav = self.generate_loop_wav(new_state);
        if let Some(wav_bytes) = wav {
            if let Ok(sink) = Sink::try_new(handle) {
                let cursor = Cursor::new(wav_bytes);
                if let Ok(source) = Decoder::new(cursor) {
                    sink.append(source.repeat_infinite());
                    sink.set_volume(match new_state {
                        MusicState::Boss        => 0.7,
                        MusicState::Combat      => 0.6,
                        MusicState::GameOver    => 0.5,
                        MusicState::Victory     => 0.8,
                        MusicState::CursedFloor => 0.55,
                        _                       => 0.45,
                    });
                    self.sink = Some(sink);
                }
            }
        }
    }

    fn generate_loop_wav(&self, state: MusicState) -> Option<Vec<u8>> {
        let samples = match state {
            MusicState::MainMenu    => synth::music_menu_loop(),
            MusicState::Exploration => synth::music_exploration_loop(self.floor_seed),
            MusicState::Combat      => synth::music_combat_loop(self.floor_seed),
            MusicState::Boss        => synth::music_boss_loop(self.floor_seed),
            MusicState::Shop        => synth::music_exploration_loop(self.floor_seed ^ 0x5555_AAAA_5A5A_A5A5),
            MusicState::CursedFloor => synth::music_cursed_loop(self.floor_seed),
            MusicState::GameOver    => return None, // stinger only, no loop
            MusicState::Victory     => return None, // stinger only, no loop
            MusicState::Silence     => return None,
        };
        Some(synth::encode_wav(&samples))
    }

    /// Update the floor seed so the next music loop has the right tonal seed.
    pub fn set_floor_seed(&mut self, seed: u64) {
        self.floor_seed = seed;
    }
}
