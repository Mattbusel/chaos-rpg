// Procedural audio synthesis for CHAOS RPG.
// Pure Rust, no external dependencies — uses only std.
// Produces raw f32 PCM samples that can be wrapped in WAV or fed to any backend.

pub const SAMPLE_RATE: u32 = 44_100;

// ── LCG RNG (deterministic, no rand dep) ─────────────────────────────────────

pub struct Lcg(u64);
impl Lcg {
    pub fn new(seed: u64) -> Self { Self(seed ^ 0x9e37_79b9_7f4a_7c15) }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        self.0
    }
    /// Uniform float in [0, 1).
    pub fn next_f32(&mut self) -> f32 { (self.next_u64() >> 33) as f32 / (1u64 << 31) as f32 }
    /// Float in [-1, 1).
    pub fn next_f32_signed(&mut self) -> f32 { self.next_f32() * 2.0 - 1.0 }
}

// ── Oscillators ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub enum Waveform {
    Sine,
    Saw,
    Square { duty: f32 },
    Triangle,
    Noise,
}

pub fn oscillator(waveform: Waveform, freq: f32, phase: f32) -> f32 {
    match waveform {
        Waveform::Sine => (phase * std::f32::consts::TAU).sin(),
        Waveform::Saw => 2.0 * (phase - phase.floor()) - 1.0,
        Waveform::Square { duty } => if (phase - phase.floor()) < duty { 1.0 } else { -1.0 },
        Waveform::Triangle => {
            let t = phase - phase.floor();
            if t < 0.5 { 4.0 * t - 1.0 } else { 3.0 - 4.0 * t }
        }
        Waveform::Noise => {
            // Deterministic per-sample noise using a fast hash of phase + freq
            let bits = (phase.to_bits() ^ freq.to_bits()).wrapping_mul(0x9e3779b9);
            let v = ((bits >> 16) & 0xffff) as f32 / 32768.0 - 1.0;
            v
        }
    }
}

/// Advance phase by one sample.
#[inline]
pub fn advance_phase(phase: f32, freq: f32) -> f32 {
    let p = phase + freq / SAMPLE_RATE as f32;
    p - p.floor()
}

// ── ADSR Envelope ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub struct Adsr {
    pub attack:  f32, // seconds
    pub decay:   f32,
    pub sustain: f32, // level 0–1
    pub release: f32,
}

impl Adsr {
    pub fn amplitude(&self, t: f32, duration: f32) -> f32 {
        let release_start = duration - self.release;
        if t < self.attack {
            t / self.attack.max(1e-6)
        } else if t < self.attack + self.decay {
            let d = (t - self.attack) / self.decay.max(1e-6);
            1.0 - d * (1.0 - self.sustain)
        } else if t < release_start {
            self.sustain
        } else if t < duration {
            let r = (t - release_start) / self.release.max(1e-6);
            self.sustain * (1.0 - r)
        } else {
            0.0
        }
    }
}

// ── Filters ───────────────────────────────────────────────────────────────────

/// First-order lowpass IIR filter state.
pub struct Lowpass {
    pub cutoff: f32,
    prev: f32,
}

impl Lowpass {
    pub fn new(cutoff_hz: f32) -> Self { Self { cutoff: cutoff_hz, prev: 0.0 } }
    pub fn process(&mut self, input: f32) -> f32 {
        let rc = 1.0 / (std::f32::consts::TAU * self.cutoff);
        let dt = 1.0 / SAMPLE_RATE as f32;
        let alpha = dt / (rc + dt);
        self.prev = self.prev + alpha * (input - self.prev);
        self.prev
    }
}

/// First-order highpass IIR filter state.
pub struct Highpass {
    pub cutoff: f32,
    prev_in: f32,
    prev_out: f32,
}

impl Highpass {
    pub fn new(cutoff_hz: f32) -> Self { Self { cutoff: cutoff_hz, prev_in: 0.0, prev_out: 0.0 } }
    pub fn process(&mut self, input: f32) -> f32 {
        let rc = 1.0 / (std::f32::consts::TAU * self.cutoff);
        let dt = 1.0 / SAMPLE_RATE as f32;
        let alpha = rc / (rc + dt);
        let out = alpha * (self.prev_out + input - self.prev_in);
        self.prev_in = input;
        self.prev_out = out;
        out
    }
}

// ── Bitcrusher ────────────────────────────────────────────────────────────────

/// Quantise to `bits` bits and downsample by `rate` factor.
pub fn bitcrush(samples: &mut Vec<f32>, bits: u8, rate: u32) {
    let levels = (1u32 << bits) as f32;
    let mut hold = 0f32;
    for (i, s) in samples.iter_mut().enumerate() {
        if i as u32 % rate == 0 {
            hold = (*s * levels).round() / levels;
        }
        *s = hold;
    }
}

// ── WAV encoder ───────────────────────────────────────────────────────────────

/// Encode mono f32 samples to a WAV byte buffer (PCM 16-bit, mono, 44100 Hz).
pub fn encode_wav(samples: &[f32]) -> Vec<u8> {
    let num_samples = samples.len() as u32;
    let byte_rate = SAMPLE_RATE * 2; // 16-bit mono
    let data_len = num_samples * 2;
    let chunk_size = 36 + data_len;
    let mut buf = Vec::with_capacity(44 + data_len as usize);

    // RIFF header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&chunk_size.to_le_bytes());
    buf.extend_from_slice(b"WAVE");

    // fmt chunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes());     // subchunk1 size
    buf.extend_from_slice(&1u16.to_le_bytes());      // PCM
    buf.extend_from_slice(&1u16.to_le_bytes());      // mono
    buf.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes());      // block align
    buf.extend_from_slice(&16u16.to_le_bytes());     // bits per sample

    // data chunk
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_len.to_le_bytes());
    for &s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        let pcm = (clamped * 32767.0) as i16;
        buf.extend_from_slice(&pcm.to_le_bytes());
    }
    buf
}

// ── Helper: generate samples for a duration ───────────────────────────────────

pub fn generate(duration_secs: f32) -> usize {
    (duration_secs * SAMPLE_RATE as f32) as usize
}

// ── Sound effects ─────────────────────────────────────────────────────────────

/// Standard melee swing.
pub fn sfx_attack() -> Vec<f32> {
    let dur = 0.18f32;
    let n = generate(dur);
    let adsr = Adsr { attack: 0.01, decay: 0.05, sustain: 0.3, release: 0.12 };
    let mut phase = 0.0f32;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        // Frequency sweep downward (whoosh)
        let freq = 300.0 - (t / dur) * 180.0;
        let s = oscillator(Waveform::Saw, freq, phase) * 0.6
              + oscillator(Waveform::Noise, freq, phase) * 0.4;
        phase = advance_phase(phase, freq);
        s * adsr.amplitude(t, dur)
    }).collect()
}

/// Heavy attack — deeper, longer.
pub fn sfx_heavy_attack() -> Vec<f32> {
    let dur = 0.35f32;
    let n = generate(dur);
    let adsr = Adsr { attack: 0.02, decay: 0.08, sustain: 0.4, release: 0.25 };
    let mut phase = 0.0f32;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let freq = 180.0 - (t / dur) * 120.0;
        let s = oscillator(Waveform::Square { duty: 0.4 }, freq, phase) * 0.5
              + oscillator(Waveform::Noise, freq, phase) * 0.5;
        phase = advance_phase(phase, freq);
        s * adsr.amplitude(t, dur)
    }).collect()
}

/// Hit impact.
pub fn sfx_hit(is_crit: bool) -> Vec<f32> {
    let dur = if is_crit { 0.25 } else { 0.12 };
    let n = generate(dur);
    let adsr = Adsr { attack: 0.002, decay: 0.04, sustain: 0.1, release: dur - 0.042 };
    let mut phase = 0.0f32;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let freq = if is_crit { 500.0 - (t / dur) * 400.0 } else { 200.0 - (t / dur) * 150.0 };
        let s = oscillator(Waveform::Noise, freq, phase);
        phase = advance_phase(phase, freq);
        s * adsr.amplitude(t, dur) * if is_crit { 1.0 } else { 0.7 }
    }).collect()
}

/// Heal chime — ascending sine notes.
pub fn sfx_heal() -> Vec<f32> {
    let dur = 0.45f32;
    let n = generate(dur);
    let freqs = [523.25f32, 659.25, 783.99]; // C5, E5, G5
    let mut out = vec![0.0f32; n];
    for (k, &f) in freqs.iter().enumerate() {
        let offset = k as f32 * 0.08;
        let adsr = Adsr { attack: 0.02, decay: 0.1, sustain: 0.4, release: 0.2 };
        let note_dur = dur - offset;
        let mut phase = 0.0f32;
        let start = (offset * SAMPLE_RATE as f32) as usize;
        for i in 0..generate(note_dur) {
            let t = i as f32 / SAMPLE_RATE as f32;
            let s = oscillator(Waveform::Sine, f, phase) * 0.4;
            phase = advance_phase(phase, f);
            if start + i < n { out[start + i] += s * adsr.amplitude(t, note_dur); }
        }
    }
    out
}

/// Player death — low descending sine with noise.
pub fn sfx_death_player() -> Vec<f32> {
    let dur = 1.2f32;
    let n = generate(dur);
    let adsr = Adsr { attack: 0.05, decay: 0.2, sustain: 0.5, release: 0.95 };
    let mut phase = 0.0f32;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let freq = 120.0 - (t / dur) * 100.0;
        let s = oscillator(Waveform::Sine, freq, phase) * 0.6
              + oscillator(Waveform::Noise, 80.0, phase) * 0.2;
        phase = advance_phase(phase, freq);
        s * adsr.amplitude(t, dur)
    }).collect()
}

/// Enemy death.
pub fn sfx_death_enemy() -> Vec<f32> {
    let dur = 0.3f32;
    let n = generate(dur);
    let adsr = Adsr { attack: 0.005, decay: 0.05, sustain: 0.2, release: 0.245 };
    let mut phase = 0.0f32;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let freq = 250.0 - (t / dur) * 200.0;
        let s = oscillator(Waveform::Saw, freq, phase) * 0.5
              + oscillator(Waveform::Noise, freq, phase) * 0.5;
        phase = advance_phase(phase, freq);
        s * adsr.amplitude(t, dur)
    }).collect()
}

/// Spell cast — ethereal rising tone.
pub fn sfx_spell(spell_index: usize) -> Vec<f32> {
    let dur = 0.4f32;
    let n = generate(dur);
    let base_freqs = [330.0f32, 392.0, 440.0, 523.25, 587.33, 659.25, 698.46, 783.99];
    let base = base_freqs[spell_index % base_freqs.len()];
    let adsr = Adsr { attack: 0.05, decay: 0.1, sustain: 0.5, release: 0.25 };
    let mut p1 = 0.0f32;
    let mut p2 = 0.0f32;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let freq = base + (t / dur) * base * 0.5; // rising
        let s = oscillator(Waveform::Sine, freq, p1) * 0.5
              + oscillator(Waveform::Triangle, freq * 2.0, p2) * 0.3;
        p1 = advance_phase(p1, freq);
        p2 = advance_phase(p2, freq * 2.0);
        s * adsr.amplitude(t, dur)
    }).collect()
}

/// Level up — triumphant two-note stinger.
pub fn sfx_level_up() -> Vec<f32> {
    let dur = 0.6f32;
    let n = generate(dur);
    let notes: [(f32, f32); 2] = [(523.25, 0.0), (783.99, 0.25)]; // C5, G5
    let mut out = vec![0.0f32; n];
    for (freq, offset) in notes {
        let adsr = Adsr { attack: 0.02, decay: 0.05, sustain: 0.6, release: 0.28 };
        let note_dur = 0.35;
        let mut phase = 0.0f32;
        let start = (offset * SAMPLE_RATE as f32) as usize;
        for i in 0..generate(note_dur) {
            let t = i as f32 / SAMPLE_RATE as f32;
            let s = oscillator(Waveform::Sine, freq, phase) * 0.5
                  + oscillator(Waveform::Triangle, freq * 2.0, phase) * 0.2;
            phase = advance_phase(phase, freq);
            let idx = start + i;
            if idx < n { out[idx] += s * adsr.amplitude(t, note_dur); }
        }
    }
    out
}

/// Short UI navigation blip.
pub fn sfx_menu_navigate() -> Vec<f32> {
    let dur = 0.05f32;
    let n = generate(dur);
    let adsr = Adsr { attack: 0.005, decay: 0.02, sustain: 0.3, release: 0.025 };
    let mut phase = 0.0f32;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let s = oscillator(Waveform::Square { duty: 0.5 }, 880.0, phase) * 0.3;
        phase = advance_phase(phase, 880.0);
        s * adsr.amplitude(t, dur)
    }).collect()
}

/// Menu confirm blip (higher pitch).
pub fn sfx_menu_confirm() -> Vec<f32> {
    let dur = 0.08f32;
    let n = generate(dur);
    let adsr = Adsr { attack: 0.005, decay: 0.03, sustain: 0.4, release: 0.045 };
    let freqs = [1046.5f32, 1318.5]; // C6, E6
    let mut out = vec![0.0f32; n];
    for (k, &f) in freqs.iter().enumerate() {
        let off = k as f32 * 0.03;
        let start = (off * SAMPLE_RATE as f32) as usize;
        let nd = dur - off;
        let mut phase = 0.0f32;
        for i in 0..generate(nd) {
            let t = i as f32 / SAMPLE_RATE as f32;
            let s = oscillator(Waveform::Sine, f, phase) * 0.35;
            phase = advance_phase(phase, f);
            let idx = start + i;
            if idx < n { out[idx] += s * adsr.amplitude(t, nd); }
        }
    }
    out
}

/// Cancel / error sound.
pub fn sfx_menu_cancel() -> Vec<f32> {
    let dur = 0.1f32;
    let n = generate(dur);
    let adsr = Adsr { attack: 0.005, decay: 0.04, sustain: 0.2, release: 0.055 };
    let mut phase = 0.0f32;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let freq = 220.0 - (t / dur) * 80.0;
        let s = oscillator(Waveform::Square { duty: 0.3 }, freq, phase) * 0.35;
        phase = advance_phase(phase, freq);
        s * adsr.amplitude(t, dur)
    }).collect()
}

/// Item pickup chime.
pub fn sfx_item_pickup() -> Vec<f32> {
    let dur = 0.2f32;
    let n = generate(dur);
    let adsr = Adsr { attack: 0.01, decay: 0.05, sustain: 0.4, release: 0.14 };
    let mut phase = 0.0f32;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let freq = 700.0 + (t / dur) * 300.0;
        let s = oscillator(Waveform::Triangle, freq, phase) * 0.4;
        phase = advance_phase(phase, freq);
        s * adsr.amplitude(t, dur)
    }).collect()
}

/// Shop entry — gentle bell.
pub fn sfx_shop_enter() -> Vec<f32> {
    let dur = 0.5f32;
    let n = generate(dur);
    let adsr = Adsr { attack: 0.01, decay: 0.1, sustain: 0.3, release: 0.39 };
    let mut p1 = 0.0f32;
    let mut p2 = 0.0f32;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let s = oscillator(Waveform::Sine, 1046.5, p1) * 0.4
              + oscillator(Waveform::Sine, 1568.0, p2) * 0.2;
        p1 = advance_phase(p1, 1046.5);
        p2 = advance_phase(p2, 1568.0);
        s * adsr.amplitude(t, dur)
    }).collect()
}

/// Trap triggered — harsh buzz.
pub fn sfx_trap(disarmed: bool) -> Vec<f32> {
    let dur = 0.25f32;
    let n = generate(dur);
    let adsr = Adsr { attack: 0.002, decay: 0.08, sustain: 0.3, release: 0.168 };
    let mut phase = 0.0f32;
    let mut samples: Vec<f32> = (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let freq = if disarmed { 440.0 } else { 110.0 + (t / dur) * 40.0 };
        let s = oscillator(Waveform::Saw, freq, phase) * 0.6
              + oscillator(Waveform::Noise, freq, phase) * 0.4;
        phase = advance_phase(phase, freq);
        s * adsr.amplitude(t, dur)
    }).collect();
    if !disarmed {
        // Add bitcrush to disarmed=false (harsh fail sound)
        bitcrush(&mut samples, 4, 2);
    }
    samples
}

/// Craft operation sound. `op_index` 0–5 selects tonal flavour.
pub fn sfx_craft(op_index: usize, success: bool) -> Vec<f32> {
    let dur = 0.4f32;
    let n = generate(dur);
    let base_freqs = [220.0f32, 293.66, 349.23, 440.0, 587.33, 698.46];
    let base = base_freqs[op_index % base_freqs.len()];
    let adsr = Adsr { attack: 0.03, decay: 0.1, sustain: 0.4, release: 0.27 };
    let mut phase = 0.0f32;
    let mut samples: Vec<f32> = (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let freq = if success {
            base + (t / dur) * base * 0.5
        } else {
            base - (t / dur) * base * 0.3
        };
        let s = oscillator(Waveform::Triangle, freq, phase) * 0.5
              + oscillator(Waveform::Sine, freq * 1.5, phase) * 0.3;
        phase = advance_phase(phase, freq);
        s * adsr.amplitude(t, dur)
    }).collect();
    if !success {
        bitcrush(&mut samples, 5, 3);
    }
    samples
}

/// Floor transition — whoosh + reverb tail.
pub fn sfx_floor_transition() -> Vec<f32> {
    let dur = 0.7f32;
    let n = generate(dur);
    let adsr = Adsr { attack: 0.05, decay: 0.1, sustain: 0.3, release: 0.55 };
    let mut lp = Lowpass::new(800.0);
    let mut phase = 0.0f32;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let freq = 80.0 + (t / dur) * 600.0;
        let raw = oscillator(Waveform::Noise, freq, phase) * 0.8
                + oscillator(Waveform::Saw, freq * 0.5, phase) * 0.2;
        phase = advance_phase(phase, freq);
        lp.process(raw) * adsr.amplitude(t, dur)
    }).collect()
}

/// Chaos engine roll sound. `engine_id` 0–9 gives each engine a distinct timbre.
/// All ~60 ms, deterministic per engine_id.
pub fn sfx_engine_roll(engine_id: u8) -> Vec<f32> {
    let dur = 0.06f32;
    let n = generate(dur);
    // Each engine: unique base freq + waveform
    let params: [(f32, Waveform); 10] = [
        (220.0, Waveform::Sine),                    // 0: Lorenz — smooth
        (349.23, Waveform::Sine),                   // 1: Fourier — harmonic
        (277.18, Waveform::Square { duty: 0.5 }),   // 2: Prime — rigid
        (293.66, Waveform::Triangle),               // 3: Riemann — angular
        (196.0, Waveform::Sine),                    // 4: Fibonacci — organic
        (261.63, Waveform::Saw),                    // 5: Mandelbrot — jagged
        (311.13, Waveform::Square { duty: 0.3 }),   // 6: Logistic — unstable
        (329.63, Waveform::Triangle),               // 7: Euler — steady
        (246.94, Waveform::Saw),                    // 8: Collatz — erratic
        (233.08, Waveform::Noise),                  // 9: Modular — chaotic
    ];
    let (base_freq, wave) = params[(engine_id % 10) as usize];
    let adsr = Adsr { attack: 0.005, decay: 0.02, sustain: 0.4, release: 0.035 };
    let mut phase = 0.0f32;
    let mut lp = Lowpass::new(3000.0);
    let out: Vec<f32> = (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let raw = oscillator(wave, base_freq, phase);
        phase = advance_phase(phase, base_freq);
        lp.process(raw) * adsr.amplitude(t, dur) * 0.5
    }).collect();
    out
}

/// Chaos cascade — layered engine rolls with small pitch offsets.
pub fn sfx_chaos_cascade(depth: u8) -> Vec<f32> {
    let layers = (depth as usize).min(5).max(1);
    let dur = 0.12f32 + layers as f32 * 0.05;
    let n = generate(dur);
    let mut out = vec![0.0f32; n];
    for k in 0..layers {
        let engine_id = k as u8;
        let offset_samples = k * (SAMPLE_RATE as usize / 40);
        let layer = sfx_engine_roll(engine_id);
        for (i, &s) in layer.iter().enumerate() {
            let idx = offset_samples + i;
            if idx < n { out[idx] += s / layers as f32; }
        }
    }
    out
}

/// Victory fanfare — bright ascending arpeggio.
pub fn sfx_victory() -> Vec<f32> {
    let notes = [523.25f32, 659.25, 783.99, 1046.5, 1318.5]; // C5 E5 G5 C6 E6
    let note_dur = 0.18f32;
    let total = generate(notes.len() as f32 * note_dur * 0.8 + 0.5);
    let mut out = vec![0.0f32; total];
    for (k, &f) in notes.iter().enumerate() {
        let offset = k as f32 * note_dur * 0.7;
        let start = (offset * SAMPLE_RATE as f32) as usize;
        let adsr = Adsr { attack: 0.01, decay: 0.05, sustain: 0.6, release: 0.12 };
        let mut phase = 0.0f32;
        for i in 0..generate(note_dur) {
            let t = i as f32 / SAMPLE_RATE as f32;
            let s = oscillator(Waveform::Sine, f, phase) * 0.4
                  + oscillator(Waveform::Triangle, f * 2.0, phase) * 0.2;
            phase = advance_phase(phase, f);
            let idx = start + i;
            if idx < out.len() { out[idx] += s * adsr.amplitude(t, note_dur); }
        }
    }
    out
}

/// Game over stinger — descending minor chord.
pub fn sfx_game_over() -> Vec<f32> {
    let notes = [392.0f32, 311.13, 261.63]; // G4, Eb4, C4 — minor
    let total = generate(1.4);
    let mut out = vec![0.0f32; total];
    for (k, &f) in notes.iter().enumerate() {
        let offset = k as f32 * 0.15;
        let start = (offset * SAMPLE_RATE as f32) as usize;
        let adsr = Adsr { attack: 0.05, decay: 0.2, sustain: 0.5, release: 0.8 };
        let nd = 1.2 - offset;
        let mut phase = 0.0f32;
        for i in 0..generate(nd) {
            let t = i as f32 / SAMPLE_RATE as f32;
            let s = oscillator(Waveform::Sine, f, phase) * 0.35
                  + oscillator(Waveform::Triangle, f * 0.5, phase) * 0.2;
            phase = advance_phase(phase, f);
            let idx = start + i;
            if idx < out.len() { out[idx] += s * adsr.amplitude(t, nd); }
        }
    }
    out
}

/// Hunger tick — low pulsing dread.
pub fn sfx_hunger() -> Vec<f32> {
    let dur = 0.5f32;
    let n = generate(dur);
    let adsr = Adsr { attack: 0.02, decay: 0.1, sustain: 0.4, release: 0.38 };
    let mut phase = 0.0f32;
    let mut lp = Lowpass::new(200.0);
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let raw = oscillator(Waveform::Sine, 55.0, phase) * 0.8
                + oscillator(Waveform::Noise, 55.0, phase) * 0.3;
        phase = advance_phase(phase, 55.0);
        lp.process(raw) * adsr.amplitude(t, dur)
    }).collect()
}

/// Boss encounter start — heavy impact.
pub fn sfx_boss_start(tier: u8) -> Vec<f32> {
    let dur = 0.8f32;
    let n = generate(dur);
    let freq = match tier { 2 => 60.0, 3 => 45.0, _ => 80.0 };
    let adsr = Adsr { attack: 0.01, decay: 0.15, sustain: 0.3, release: 0.64 };
    let mut p1 = 0.0f32;
    let mut p2 = 0.0f32;
    let mut lp = Lowpass::new(400.0);
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let raw = oscillator(Waveform::Square { duty: 0.5 }, freq, p1) * 0.5
                + oscillator(Waveform::Noise, freq * 2.0, p2) * 0.5;
        p1 = advance_phase(p1, freq);
        p2 = advance_phase(p2, freq * 2.0);
        lp.process(raw) * adsr.amplitude(t, dur)
    }).collect()
}

/// Nemesis spawned — ominous low chord.
pub fn sfx_nemesis_spawned() -> Vec<f32> {
    let notes = [110.0f32, 138.59, 164.81]; // A2, Db3, E3 — dark tritone-ish
    let dur = 1.0f32;
    let total = generate(dur);
    let mut out = vec![0.0f32; total];
    for (k, &f) in notes.iter().enumerate() {
        let off = k as f32 * 0.08;
        let start = (off * SAMPLE_RATE as f32) as usize;
        let nd = dur - off;
        let adsr = Adsr { attack: 0.08, decay: 0.2, sustain: 0.5, release: 0.52 };
        let mut phase = 0.0f32;
        for i in 0..generate(nd) {
            let t = i as f32 / SAMPLE_RATE as f32;
            let s = oscillator(Waveform::Saw, f, phase) * 0.25;
            phase = advance_phase(phase, f);
            let idx = start + i;
            if idx < out.len() { out[idx] += s * adsr.amplitude(t, nd); }
        }
    }
    out
}

/// Boon selected — crystalline shimmer.
pub fn sfx_boon_selected() -> Vec<f32> {
    let dur = 0.5f32;
    let n = generate(dur);
    let adsr = Adsr { attack: 0.01, decay: 0.1, sustain: 0.5, release: 0.39 };
    let mut p1 = 0.0f32;
    let mut p2 = 0.0f32;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let freq = 1200.0 + (t / dur) * 400.0;
        let s = oscillator(Waveform::Sine, freq, p1) * 0.35
              + oscillator(Waveform::Triangle, freq * 1.5, p2) * 0.25;
        p1 = advance_phase(p1, freq);
        p2 = advance_phase(p2, freq * 1.5);
        s * adsr.amplitude(t, dur)
    }).collect()
}

/// Item volatility reroll — glitchy pop.
pub fn sfx_volatility_reroll() -> Vec<f32> {
    let dur = 0.2f32;
    let n = generate(dur);
    let adsr = Adsr { attack: 0.005, decay: 0.05, sustain: 0.3, release: 0.145 };
    let mut phase = 0.0f32;
    let mut samples: Vec<f32> = (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let freq = 400.0 + (t / dur) * 600.0 - (t / dur).powi(2) * 800.0;
        let s = oscillator(Waveform::Square { duty: 0.5 }, freq, phase) * 0.5
              + oscillator(Waveform::Noise, freq, phase) * 0.3;
        phase = advance_phase(phase, freq);
        s * adsr.amplitude(t, dur)
    }).collect();
    bitcrush(&mut samples, 6, 2);
    samples
}

// ── Music generators ──────────────────────────────────────────────────────────

/// Generate one loop of exploration ambient music (~4 seconds).
/// `seed` makes it deterministic per floor.
pub fn music_exploration_loop(seed: u64) -> Vec<f32> {
    let dur = 4.0f32;
    let n = generate(dur);
    let mut rng = Lcg::new(seed);
    let mut out = vec![0.0f32; n];
    let mut lp = Lowpass::new(600.0);
    let mut hp = Highpass::new(60.0);

    // Bass drone
    let bass_freq = [55.0f32, 65.41, 73.42, 82.41][(rng.next_u64() % 4) as usize];
    let mut bp = 0.0f32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let adsr_val = 0.6 + 0.4 * (t * std::f32::consts::TAU * 0.25).sin();
        let s = oscillator(Waveform::Sine, bass_freq, bp) * 0.3 * adsr_val;
        bp = advance_phase(bp, bass_freq);
        out[i] += s;
    }

    // Sparse pads — 3 notes staggered
    let pad_freqs = [
        bass_freq * 2.0,
        bass_freq * 2.5,
        bass_freq * 3.0 + rng.next_f32() * 20.0,
    ];
    for (k, &pf) in pad_freqs.iter().enumerate() {
        let off = k as f32 * (dur / 4.0);
        let start = (off * SAMPLE_RATE as f32) as usize;
        let nd = dur - off;
        let adsr = Adsr { attack: 0.3, decay: 0.4, sustain: 0.4, release: nd - 0.7 };
        let mut phase = 0.0f32;
        for i in 0..generate(nd) {
            let t = i as f32 / SAMPLE_RATE as f32;
            let s = oscillator(Waveform::Sine, pf, phase) * 0.15;
            phase = advance_phase(phase, pf);
            let idx = start + i;
            if idx < n { out[idx] += s * adsr.amplitude(t, nd); }
        }
    }

    // Subtle noise texture
    let mut noise_phase = 0.0f32;
    for i in 0..n {
        let s = oscillator(Waveform::Noise, 200.0, noise_phase) * 0.04;
        noise_phase = advance_phase(noise_phase, 200.0);
        out[i] += lp.process(s);
    }
    // Highpass to remove sub-bass mud
    for s in &mut out { *s = hp.process(*s); }
    out
}

/// Generate one loop of combat music (~2 seconds, energetic).
pub fn music_combat_loop(seed: u64) -> Vec<f32> {
    let dur = 2.0f32;
    let n = generate(dur);
    let mut rng = Lcg::new(seed ^ 0xdead_beef);
    let mut out = vec![0.0f32; n];

    // Bass pulse — Triangle is warmer than Square/Saw
    let bass_freq = 110.0f32;
    let beat_period = dur / 4.0;
    for beat in 0..4usize {
        let start = (beat as f32 * beat_period * SAMPLE_RATE as f32) as usize;
        let nd = beat_period * 0.55;
        let adsr = Adsr { attack: 0.02, decay: 0.06, sustain: 0.35, release: nd - 0.08 };
        let mut phase = 0.0f32;
        for i in 0..generate(nd) {
            let t = i as f32 / SAMPLE_RATE as f32;
            let s = oscillator(Waveform::Triangle, bass_freq, phase) * 0.28
                  + oscillator(Waveform::Sine, bass_freq * 2.0, phase) * 0.12;
            phase = advance_phase(phase, bass_freq);
            let idx = start + i;
            if idx < n { out[idx] += s * adsr.amplitude(t, nd); }
        }
    }

    // Softer texture — quiet sine stabs instead of harsh saw
    for _ in 0..4 {
        let off = rng.next_f32() * dur;
        let freq = 220.0 + rng.next_f32() * 330.0; // lower, less shrill
        let start = (off * SAMPLE_RATE as f32) as usize;
        let nd = 0.07;
        let adsr = Adsr { attack: 0.01, decay: 0.03, sustain: 0.2, release: 0.03 };
        let mut phase = 0.0f32;
        for i in 0..generate(nd) {
            let t = i as f32 / SAMPLE_RATE as f32;
            let s = oscillator(Waveform::Sine, freq, phase) * 0.10;
            phase = advance_phase(phase, freq);
            let idx = start + i;
            if idx < n { out[idx] += s * adsr.amplitude(t, nd); }
        }
    }
    out
}

/// Generate boss music loop (~3 seconds, heavy).
pub fn music_boss_loop(seed: u64) -> Vec<f32> {
    let dur = 3.0f32;
    let n = generate(dur);
    let mut rng = Lcg::new(seed ^ 0xb055_b055);
    let mut out = vec![0.0f32; n];
    let mut lp = Lowpass::new(800.0);

    // Heavy sub bass — Triangle instead of Square (softer harmonic profile)
    let sub = 55.0f32;
    let mut bp = 0.0f32;
    for i in 0..n {
        let s = oscillator(Waveform::Triangle, sub, bp) * 0.28;
        bp = advance_phase(bp, sub);
        out[i] += lp.process(s);
    }

    // Mid layer — Sine + slow LFO tremolo instead of raw Saw
    let mid_freq = 220.0f32;
    let mut mp = 0.0f32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let tremolo = 0.7 + 0.3 * (t * std::f32::consts::TAU * 1.5).sin();
        let s = oscillator(Waveform::Sine, mid_freq, mp) * 0.18 * tremolo;
        mp = advance_phase(mp, mid_freq);
        out[i] += s;
    }

    // Percussive noise bursts — quieter
    let beat_period = dur / 6.0;
    for beat in 0..6usize {
        let start = (beat as f32 * beat_period * SAMPLE_RATE as f32) as usize;
        let nd = 0.04;
        let adsr = Adsr { attack: 0.002, decay: 0.015, sustain: 0.15, release: 0.023 };
        let mut phase = 0.0f32;
        for i in 0..generate(nd) {
            let t = i as f32 / SAMPLE_RATE as f32;
            let s = oscillator(Waveform::Noise, 120.0 + rng.next_f32() * 60.0, phase) * 0.22;
            phase = advance_phase(phase, 120.0);
            let idx = start + i;
            if idx < n { out[idx] += s * adsr.amplitude(t, nd); }
        }
    }
    // No bitcrush — was too harsh
    out
}

/// Generate main menu music loop (~5 seconds, atmospheric).
pub fn music_menu_loop() -> Vec<f32> {
    let dur = 5.0f32;
    let n = generate(dur);
    let mut out = vec![0.0f32; n];
    let mut lp = Lowpass::new(500.0);

    // Slow pulsing drone
    let notes = [130.81f32, 164.81, 196.0, 130.81]; // C3 E3 G3 C3
    let note_dur = dur / notes.len() as f32;
    for (k, &f) in notes.iter().enumerate() {
        let offset = k as f32 * note_dur;
        let start = (offset * SAMPLE_RATE as f32) as usize;
        let adsr = Adsr { attack: 0.2, decay: 0.3, sustain: 0.5, release: note_dur - 0.5 };
        let mut p1 = 0.0f32;
        let mut p2 = 0.0f32;
        for i in 0..generate(note_dur) {
            let t = i as f32 / SAMPLE_RATE as f32;
            let s = oscillator(Waveform::Sine, f, p1) * 0.3
                  + oscillator(Waveform::Triangle, f * 2.01, p2) * 0.15;
            p1 = advance_phase(p1, f);
            p2 = advance_phase(p2, f * 2.01);
            let idx = start + i;
            if idx < n { out[idx] += lp.process(s) * adsr.amplitude(t, note_dur); }
        }
    }
    out
}

/// Generate cursed floor ambient loop (~3 seconds, dissonant but not clipping).
pub fn music_cursed_loop(seed: u64) -> Vec<f32> {
    let dur = 3.0f32;
    let n = generate(dur);
    let mut rng = Lcg::new(seed ^ 0xC0_DE_DEAD_BEEF_0001);
    let mut out = music_exploration_loop(seed);
    out.resize(n, 0.0);

    // Overlay dissonant tritone pads
    let base = 110.0f32;
    let dissonant = base * (2.0f32).powf(6.0 / 12.0); // tritone
    let mut p1 = 0.0f32;
    let mut p2 = 0.0f32;
    let adsr = Adsr { attack: 0.5, decay: 0.5, sustain: 0.5, release: dur - 1.0 };
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let s = oscillator(Waveform::Sine, dissonant, p1) * 0.14
              + oscillator(Waveform::Triangle, base * 0.5, p2) * 0.10;
        p1 = advance_phase(p1, dissonant);
        p2 = advance_phase(p2, base * 0.5);
        out[i] += s * adsr.amplitude(t, dur);
    }

    // Subtle glitches — attenuate rather than invert (no clipping)
    for _ in 0..3 {
        let pos = (rng.next_f32() * n as f32) as usize;
        let len = (rng.next_f32() * 0.015 * SAMPLE_RATE as f32) as usize;
        for j in pos..(pos + len).min(n) {
            out[j] *= 0.2; // quiet dropout rather than invert-clip
        }
    }

    // Light bitcrush for texture (was 7 bits / rate 2 — now 12 bits / rate 1)
    bitcrush(&mut out, 12, 1);
    // Final clamp to prevent any residual clipping
    for s in &mut out { *s = s.clamp(-1.0, 1.0); }
    out
}

// ── CHILL MUSIC VARIANTS ──────────────────────────────────────────────────────
// Longer loops with baked-in evolution: starts sparse, builds, breathes back.
// All amplitudes deliberately lower than Classic — designed for background play.

fn pentatonic_scale(root: f32) -> [f32; 5] {
    // Major pentatonic: root, M2, M3, P5, M6
    [root, root * 1.122, root * 1.260, root * 1.498, root * 1.682]
}

/// 16-second evolving ambient exploration loop (Chill vibe).
/// Phases: 0=bare drone → 1=+pad → 2=+melody → 3=breathe back.
pub fn music_exploration_chill(seed: u64) -> Vec<f32> {
    let phase_dur = 4.0f32;
    let n_total = generate(phase_dur * 4.0);
    let mut out = vec![0.0f32; n_total];
    let mut rng = Lcg::new(seed);

    let roots = [55.0f32, 61.74, 65.41, 73.42]; // A1 B1 C2 D2
    let bass_root = roots[(rng.next_u64() % 4) as usize];
    let scale = pentatonic_scale(bass_root * 2.0);
    let mut lp = Lowpass::new(500.0);

    for phase in 0u32..4 {
        let start = (phase as f32 * phase_dur * SAMPLE_RATE as f32) as usize;
        let n = generate(phase_dur);
        // Volume envelope: sparse → full → spare
        let vol = [0.55f32, 0.75, 1.0, 0.65][phase as usize];

        // Bass drone — always present, gentle sine breathing
        {
            let mut bp = 0.0f32;
            for i in 0..n {
                let t = i as f32 / SAMPLE_RATE as f32;
                let breath = 0.75 + 0.25 * (t * std::f32::consts::TAU * 0.25).sin();
                let s = oscillator(Waveform::Sine, bass_root, bp) * 0.20 * breath * vol;
                bp = advance_phase(bp, bass_root);
                let idx = start + i;
                if idx < n_total { out[idx] += s; }
            }
        }

        // Soft pad (5th above bass) — phase 1+
        if phase >= 1 {
            let pad_freq = bass_root * 3.0; // octave + fifth
            let adsr = Adsr { attack: 0.9, decay: 0.6, sustain: 0.35, release: phase_dur - 1.5 };
            let mut pp = 0.0f32;
            for i in 0..n {
                let t = i as f32 / SAMPLE_RATE as f32;
                let s = (oscillator(Waveform::Sine, pad_freq, pp) * 0.55
                       + oscillator(Waveform::Triangle, pad_freq * 1.498, pp) * 0.25)
                       * 0.09 * vol * adsr.amplitude(t, phase_dur);
                pp = advance_phase(pp, pad_freq);
                let idx = start + i;
                if idx < n_total { out[idx] += s; }
            }
        }

        // Sparse pentatonic melody — phase 2 only (peak density)
        if phase == 2 {
            let note_offsets = [0.3f32, 1.0, 1.9, 2.7, 3.4];
            for &note_t in &note_offsets {
                let fidx = (rng.next_u64() % 5) as usize;
                let freq = scale[fidx];
                let ns = start + (note_t * SAMPLE_RATE as f32) as usize;
                let nd = 0.55f32;
                let adsr = Adsr { attack: 0.06, decay: 0.14, sustain: 0.28, release: 0.35 };
                let mut np = 0.0f32;
                for i in 0..generate(nd) {
                    let t = i as f32 / SAMPLE_RATE as f32;
                    let s = oscillator(Waveform::Sine, freq, np) * 0.12 * vol
                          * adsr.amplitude(t, nd);
                    np = advance_phase(np, freq);
                    let idx = ns + i;
                    if idx < n_total { out[idx] += s; }
                }
            }
        }

        // Very quiet filtered noise texture — only at peak
        if phase == 2 {
            let mut np = 0.0f32;
            for i in 0..n {
                let raw = oscillator(Waveform::Noise, 160.0, np) * 0.018;
                np = advance_phase(np, 160.0);
                let idx = start + i;
                if idx < n_total { out[idx] += lp.process(raw); }
            }
        }
    }

    let mut hp = Highpass::new(35.0);
    for s in &mut out { *s = hp.process(*s).clamp(-1.0, 1.0); }
    out
}

/// 8-second evolving combat loop (Chill vibe).
/// Phase 0: gentle triangle pulse. Phase 1: adds light pentatonic arpeggio.
pub fn music_combat_chill(seed: u64) -> Vec<f32> {
    let phase_dur = 4.0f32;
    let n_total = generate(phase_dur * 2.0);
    let mut out = vec![0.0f32; n_total];
    let mut rng = Lcg::new(seed ^ 0xC0BA_2000);

    let bass_freq = 110.0f32;

    for phase in 0u32..2 {
        let start = (phase as f32 * phase_dur * SAMPLE_RATE as f32) as usize;
        let n = generate(phase_dur);
        let vol = if phase == 0 { 0.75f32 } else { 1.0 };

        // Triangle bass pulse — 4 beats per phase
        let beat_period = phase_dur / 4.0;
        for beat in 0..4usize {
            let bs = start + (beat as f32 * beat_period * SAMPLE_RATE as f32) as usize;
            let nd = beat_period * 0.52;
            let adsr = Adsr { attack: 0.03, decay: 0.07, sustain: 0.28, release: nd - 0.10 };
            let mut pp = 0.0f32;
            for i in 0..generate(nd) {
                let t = i as f32 / SAMPLE_RATE as f32;
                let s = oscillator(Waveform::Triangle, bass_freq, pp) * 0.22 * vol
                      + oscillator(Waveform::Sine, bass_freq * 2.0, pp) * 0.09 * vol;
                pp = advance_phase(pp, bass_freq);
                let idx = bs + i;
                if idx < n_total { out[idx] += s * adsr.amplitude(t, nd); }
            }
        }

        // Phase 1: gentle ascending arpeggio
        if phase == 1 {
            let root = 220.0f32;
            let arpegg = [root, root * 1.26, root * 1.498, root * 2.0];
            let note_dur = 0.18f32;
            for (k, &freq) in arpegg.iter().enumerate() {
                for rep in 0..4usize {
                    let note_t = k as f32 * 0.22 + rep as f32 * 1.0;
                    if note_t + note_dur > phase_dur { continue; }
                    let ns = start + (note_t * SAMPLE_RATE as f32) as usize;
                    let adsr = Adsr { attack: 0.02, decay: 0.05, sustain: 0.25, release: 0.11 };
                    let mut np = 0.0f32;
                    for i in 0..generate(note_dur) {
                        let t = i as f32 / SAMPLE_RATE as f32;
                        let s = oscillator(Waveform::Sine, freq, np) * 0.09 * adsr.amplitude(t, note_dur);
                        np = advance_phase(np, freq);
                        let idx = ns + i;
                        if idx < n_total { out[idx] += s; }
                    }
                }
            }
            // Off-beat hi-freq tick for rhythm
            for beat in 0..4usize {
                let tick_t = (beat as f32 + 0.5) * beat_period;
                if tick_t >= phase_dur { continue; }
                let ts = start + (tick_t * SAMPLE_RATE as f32) as usize;
                let nd = 0.03f32;
                let adsr = Adsr { attack: 0.002, decay: 0.01, sustain: 0.1, release: 0.018 };
                let mut tp = 0.0f32;
                for i in 0..generate(nd) {
                    let t = i as f32 / SAMPLE_RATE as f32;
                    let s = oscillator(Waveform::Triangle, 880.0, tp) * 0.06 * adsr.amplitude(t, nd);
                    tp = advance_phase(tp, 880.0);
                    let idx = ts + i;
                    if idx < n_total { out[idx] += s; }
                }
            }
        }
        let _ = (n, rng.next_f32()); // suppress unused warnings
    }

    for s in &mut out { *s = s.clamp(-1.0, 1.0); }
    out
}

/// 12-second boss loop (Chill vibe). Tense but clean — no bitcrush, no harsh waves.
pub fn music_boss_chill(seed: u64) -> Vec<f32> {
    let dur = 12.0f32;
    let n = generate(dur);
    let mut out = vec![0.0f32; n];
    let mut rng = Lcg::new(seed ^ 0xB055_C222);
    let mut lp = Lowpass::new(280.0);

    // Pulsing sub bass — sine, breathes every 1.5s
    let sub = 55.0f32;
    let mut bp = 0.0f32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let pulse = (0.5 + 0.5 * (t * std::f32::consts::TAU / 1.5).sin()).powi(2);
        let raw = oscillator(Waveform::Sine, sub, bp) * 0.24 * pulse;
        bp = advance_phase(bp, sub);
        out[i] += lp.process(raw);
    }

    // Ominous minor chord that slowly swells in
    let root = 110.0f32;
    let chord = [root, root * 1.189, root * 1.498]; // natural minor triad
    for (ci, &freq) in chord.iter().enumerate() {
        let attack_offset = ci as f32 * 0.8;
        let adsr = Adsr {
            attack: 1.5 + attack_offset,
            decay: 0.8,
            sustain: 0.4,
            release: dur - 2.3 - attack_offset,
        };
        let mut pp = 0.0f32;
        for i in 0..n {
            let t = i as f32 / SAMPLE_RATE as f32;
            let s = oscillator(Waveform::Sine, freq, pp) * 0.09 * adsr.amplitude(t, dur);
            pp = advance_phase(pp, freq);
            out[i] += s;
        }
    }

    // Sparse irregular taps (tension markers)
    let tap_times = [0.8f32, 2.5, 4.2, 6.0, 7.8, 9.5, 11.1];
    for &tt in &tap_times {
        let ns = (tt * SAMPLE_RATE as f32) as usize;
        let nd = 0.05f32;
        let adsr = Adsr { attack: 0.002, decay: 0.018, sustain: 0.08, release: 0.030 };
        let freq = 70.0 + rng.next_f32() * 35.0;
        let mut pp = 0.0f32;
        for i in 0..generate(nd) {
            let t = i as f32 / SAMPLE_RATE as f32;
            let s = oscillator(Waveform::Noise, freq, pp) * 0.16 * adsr.amplitude(t, nd);
            pp = advance_phase(pp, freq);
            let idx = ns + i;
            if idx < n { out[idx] += s; }
        }
    }

    // Rising tension arpeggio (builds over the 12 seconds)
    let arp = [220.0f32, 261.63, 311.13, 349.23, 415.30, 493.88];
    for (k, &freq) in arp.iter().enumerate() {
        let tt = 1.5 + k as f32 * 1.6;
        if tt >= dur { break; }
        let ns = (tt * SAMPLE_RATE as f32) as usize;
        let nd = 0.50f32;
        let vol = 0.05 + k as f32 * 0.008;
        let adsr = Adsr { attack: 0.06, decay: 0.12, sustain: 0.35, release: 0.32 };
        let mut pp = 0.0f32;
        for i in 0..generate(nd) {
            let t = i as f32 / SAMPLE_RATE as f32;
            let s = oscillator(Waveform::Triangle, freq, pp) * vol * adsr.amplitude(t, nd);
            pp = advance_phase(pp, freq);
            let idx = ns + i;
            if idx < n { out[idx] += s; }
        }
    }

    for s in &mut out { *s = s.clamp(-1.0, 1.0); }
    out
}

/// 8-second minimal bass drone (Minimal vibe). Just a quiet sub hum.
pub fn music_minimal_drone(seed: u64) -> Vec<f32> {
    let dur = 8.0f32;
    let n = generate(dur);
    let roots = [55.0f32, 61.74, 65.41, 73.42];
    let freq = roots[(seed as usize) % 4];
    let mut lp = Lowpass::new(350.0);
    let mut bp = 0.0f32;
    (0..n).map(|i| {
        let t = i as f32 / SAMPLE_RATE as f32;
        let breath = 0.80 + 0.20 * (t * std::f32::consts::TAU * 0.125).sin();
        let raw = oscillator(Waveform::Sine, freq, bp) * 0.15 * breath;
        bp = advance_phase(bp, freq);
        lp.process(raw).clamp(-1.0, 1.0)
    }).collect()
}
