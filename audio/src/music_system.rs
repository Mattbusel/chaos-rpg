// Music state machine — manages looping music layers.
// Transitions are handled by stopping the old sink and starting a new one.
// Loops are implemented by repeating the synthesized buffer.

use crate::SoundBank;
use chaos_rpg_core::audio_events::{MusicState, MusicVibe};
use chaos_rpg_core::audio_synth as synth;
use rodio::{Decoder, OutputStreamHandle, Sink, Source};
use std::io::Cursor;

pub struct MusicSystem {
    current_state: MusicState,
    sink: Option<Sink>,
    floor_seed: u64,
    vibe: MusicVibe,
}

impl MusicSystem {
    pub fn new(handle: &OutputStreamHandle, bank: &SoundBank) -> Self {
        let mut sys = Self {
            current_state: MusicState::Silence,
            sink: None,
            floor_seed: 0,
            vibe: MusicVibe::Chill,
        };
        sys.transition_to(MusicState::MainMenu, handle, bank);
        sys
    }

    pub fn set_vibe(&mut self, vibe: MusicVibe, handle: &OutputStreamHandle, bank: &SoundBank) {
        if vibe == self.vibe { return; }
        let state = self.current_state;
        self.vibe = vibe;
        // Force reload by resetting current_state
        self.current_state = MusicState::Silence;
        self.transition_to(state, handle, bank);
    }

    pub fn transition_to(&mut self, new_state: MusicState, handle: &OutputStreamHandle, _bank: &SoundBank) {
        if new_state == self.current_state { return; }

        // Stop the old sink (drop = stop)
        self.sink = None;
        self.current_state = new_state;

        // Off vibe: no music at all
        if self.vibe == MusicVibe::Off { return; }

        let wav = self.generate_loop_wav(new_state);
        if let Some(wav_bytes) = wav {
            if let Ok(sink) = Sink::try_new(handle) {
                let cursor = Cursor::new(wav_bytes);
                if let Ok(source) = Decoder::new(cursor) {
                    sink.append(source.repeat_infinite());
                    // Chill/Minimal are quieter than Classic
                    let vol_scale = match self.vibe {
                        MusicVibe::Chill   => 0.55f32,
                        MusicVibe::Classic => 0.80,
                        MusicVibe::Minimal => 0.40,
                        MusicVibe::Off     => 0.0,
                    };
                    sink.set_volume(vol_scale * match new_state {
                        MusicState::Boss        => 0.42,
                        MusicState::Combat      => 0.38,
                        MusicState::GameOver    => 0.35,
                        MusicState::Victory     => 0.45,
                        MusicState::CursedFloor => 0.36,
                        _                       => 0.30,
                    });
                    self.sink = Some(sink);
                }
            }
        }
    }

    fn generate_loop_wav(&self, state: MusicState) -> Option<Vec<u8>> {
        let samples = match self.vibe {
            MusicVibe::Off => return None,

            MusicVibe::Minimal => match state {
                MusicState::Silence | MusicState::GameOver | MusicState::Victory => return None,
                _ => synth::music_minimal_drone(self.floor_seed),
            },

            MusicVibe::Chill => match state {
                MusicState::MainMenu    => synth::music_menu_loop(),
                MusicState::Exploration | MusicState::Shop => synth::music_exploration_chill(self.floor_seed),
                MusicState::Combat      => synth::music_combat_chill(self.floor_seed),
                MusicState::Boss        => synth::music_boss_chill(self.floor_seed),
                MusicState::CursedFloor => synth::music_cursed_loop(self.floor_seed), // already fixed
                MusicState::GameOver | MusicState::Victory | MusicState::Silence => return None,
            },

            MusicVibe::Classic => match state {
                MusicState::MainMenu    => synth::music_menu_loop(),
                MusicState::Exploration => synth::music_exploration_loop(self.floor_seed),
                MusicState::Combat      => synth::music_combat_loop(self.floor_seed),
                MusicState::Boss        => synth::music_boss_loop(self.floor_seed),
                MusicState::Shop        => synth::music_exploration_loop(self.floor_seed ^ 0x5555_AAAA_5A5A_A5A5),
                MusicState::CursedFloor => synth::music_cursed_loop(self.floor_seed),
                MusicState::GameOver | MusicState::Victory | MusicState::Silence => return None,
            },
        };
        Some(synth::encode_wav(&samples))
    }

    /// Update the floor seed so the next music loop has the right tonal seed.
    pub fn set_floor_seed(&mut self, seed: u64) {
        self.floor_seed = seed;
    }
}
