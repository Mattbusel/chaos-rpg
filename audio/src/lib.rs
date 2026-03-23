// CHAOS RPG Audio — rodio-based native audio backend.
// Uses only synthesized sounds from chaos_rpg_core::audio_synth.
// Fully optional: if no audio device is available, all calls are silent no-ops.

mod sound_bank;
mod music_system;

pub use sound_bank::SoundBank;
pub use music_system::MusicSystem;

use chaos_rpg_core::audio_events::{AudioEvent, MusicState};
use std::sync::mpsc::{self, Sender};
use std::thread;

/// The main audio system. Create once at startup, then call `emit()` from the game loop.
pub struct AudioSystem {
    tx: Sender<AudioMsg>,
}

enum AudioMsg {
    Sfx(AudioEvent),
    SetMusicState(MusicState),
    Stop,
}

impl AudioSystem {
    /// Initialise the audio system. Returns `None` silently if no audio device exists.
    pub fn try_new() -> Option<Self> {
        let (tx, rx) = mpsc::channel::<AudioMsg>();

        // Probe for audio device availability before spawning
        if rodio::OutputStream::try_default().is_err() {
            return None;
        }

        thread::spawn(move || {
            // Create the stream on the audio thread (OutputStream is !Send)
            let Ok((_stream, stream_handle)) = rodio::OutputStream::try_default() else { return; };
            let bank = SoundBank::new();
            let mut music = MusicSystem::new(&stream_handle, &bank);
            for msg in rx {
                match msg {
                    AudioMsg::Sfx(ev) => bank.play(&stream_handle, &ev),
                    AudioMsg::SetMusicState(state) => music.transition_to(state, &stream_handle, &bank),
                    AudioMsg::Stop => break,
                }
            }
        });

        Some(Self { tx })
    }

    /// Emit an audio event from the game. Non-blocking — queued to the audio thread.
    pub fn emit(&self, event: AudioEvent) {
        // Derive music state changes from key events
        let music_state = match &event {
            AudioEvent::FloorEntered { .. } => Some(MusicState::Exploration),
            AudioEvent::BossEncounterStart { .. } | AudioEvent::GauntletStart => Some(MusicState::Boss),
            AudioEvent::PlayerAttack | AudioEvent::EnemyAttack | AudioEvent::DamageDealt { .. } => {
                Some(MusicState::Combat)
            }
            AudioEvent::ShopEntered => Some(MusicState::Shop),
            AudioEvent::GameOver => Some(MusicState::GameOver),
            AudioEvent::Victory => Some(MusicState::Victory),
            AudioEvent::CursedFloorActivated => Some(MusicState::CursedFloor),
            _ => None,
        };

        let _ = self.tx.send(AudioMsg::Sfx(event));
        if let Some(state) = music_state {
            let _ = self.tx.send(AudioMsg::SetMusicState(state));
        }
    }

    /// Explicitly set music state without triggering a SFX.
    pub fn set_music_state(&self, state: MusicState) {
        let _ = self.tx.send(AudioMsg::SetMusicState(state));
    }
}

impl Drop for AudioSystem {
    fn drop(&mut self) {
        let _ = self.tx.send(AudioMsg::Stop);
    }
}
