//! Audio-reactive visual system — makes the game pulse with its own music.
//!
//! Runs FFT on the procedural music output every frame, extracts frequency
//! band energies (bass/mid/high), detects beats, and smooths an amplitude
//! envelope. These values drive visual parameters across all game systems:
//!
//! - **Bass energy** → chaos field particle speed multiplier
//! - **Beat detected** → camera FOV pulse + screen border flash
//! - **Mid energy** → ambient force field strength oscillation
//! - **High energy** → entity emission pulse (glow on hi-hats)
//! - **Envelope** → vignette intensity (louder = less vignette)
//!
//! The renderer reads `AudioReactiveOutput` each frame and applies the
//! visual modulations on top of the normal scene.

use glam::Vec4;
use std::f32::consts::PI;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Constants
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// FFT window size (must be power of 2).
const FFT_SIZE: usize = 1024;
/// Audio sample rate (must match engine).
const SAMPLE_RATE: f32 = 48000.0;
/// Hz per FFT bin.
const HZ_PER_BIN: f32 = SAMPLE_RATE / FFT_SIZE as f32;

// Frequency band boundaries (Hz).
const BASS_LOW: f32 = 20.0;
const BASS_HIGH: f32 = 200.0;
const MID_LOW: f32 = 200.0;
const MID_HIGH: f32 = 2000.0;
const HIGH_LOW: f32 = 2000.0;
const HIGH_HIGH: f32 = 20000.0;

// Smoothing coefficients (exponential moving average).
const BASS_SMOOTH: f32 = 0.15;
const MID_SMOOTH: f32 = 0.20;
const HIGH_SMOOTH: f32 = 0.25;
const ENVELOPE_SMOOTH: f32 = 0.10;

// Beat detection.
const BEAT_THRESHOLD: f32 = 1.4;
const BEAT_COOLDOWN: f32 = 0.18;  // seconds between beats

// Visual mapping parameters.
const PARTICLE_SPEED_BASS_SCALE: f32 = 0.5;
const FOV_BEAT_PULSE: f32 = -0.3;  // degrees (negative = zoom in)
const FOV_DECAY_RATE: f32 = 12.0;   // degrees/sec return speed
const FORCE_FIELD_MID_SCALE: f32 = 0.6;
const EMISSION_HIGH_SCALE: f32 = 0.8;
const VIGNETTE_BASE: f32 = 0.5;
const VIGNETTE_LOUD_REDUCTION: f32 = 0.3;
const BORDER_FLASH_OPACITY: f32 = 0.10;
const BORDER_FLASH_DURATION: f32 = 1.0 / 60.0;  // 1 frame at 60fps

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// FFT helpers (inline radix-2 to avoid coupling to proof-engine DSP module)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// In-place Cooley-Tukey radix-2 DIT FFT.
fn fft_in_place(re: &mut [f32], im: &mut [f32]) {
    let n = re.len();
    debug_assert!(n.is_power_of_two());

    // Bit-reversal permutation
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }

    // Butterfly stages
    let mut len = 2;
    while len <= n {
        let half = len / 2;
        let angle = -2.0 * PI / len as f32;
        let wn_re = angle.cos();
        let wn_im = angle.sin();

        let mut start = 0;
        while start < n {
            let mut w_re = 1.0_f32;
            let mut w_im = 0.0_f32;
            for k in 0..half {
                let a = start + k;
                let b = start + k + half;

                let t_re = w_re * re[b] - w_im * im[b];
                let t_im = w_re * im[b] + w_im * re[b];

                re[b] = re[a] - t_re;
                im[b] = im[a] - t_im;
                re[a] += t_re;
                im[a] += t_im;

                let new_w_re = w_re * wn_re - w_im * wn_im;
                let new_w_im = w_re * wn_im + w_im * wn_re;
                w_re = new_w_re;
                w_im = new_w_im;
            }
            start += len;
        }
        len <<= 1;
    }
}

/// Compute magnitude spectrum from a real signal.
fn magnitude_spectrum(signal: &[f32]) -> Vec<f32> {
    let n = FFT_SIZE;
    let mut re = vec![0.0f32; n];
    let mut im = vec![0.0f32; n];

    // Copy signal with Hann window
    let samples = signal.len().min(n);
    for i in 0..samples {
        let window = 0.5 * (1.0 - (2.0 * PI * i as f32 / (n - 1) as f32).cos());
        re[i] = signal[i] * window;
    }

    fft_in_place(&mut re, &mut im);

    // Magnitude of first N/2+1 bins
    let bins = n / 2 + 1;
    let mut mag = Vec::with_capacity(bins);
    for i in 0..bins {
        mag.push((re[i] * re[i] + im[i] * im[i]).sqrt());
    }
    mag
}

/// Sum energy in a frequency range from the magnitude spectrum.
fn band_energy(mag: &[f32], low_hz: f32, high_hz: f32) -> f32 {
    let bin_low = (low_hz / HZ_PER_BIN).round() as usize;
    let bin_high = (high_hz / HZ_PER_BIN).round() as usize;
    let bin_low = bin_low.clamp(0, mag.len() - 1);
    let bin_high = bin_high.clamp(bin_low, mag.len() - 1);

    if bin_high <= bin_low {
        return 0.0;
    }

    let sum: f32 = mag[bin_low..=bin_high].iter().map(|m| m * m).sum();
    (sum / (bin_high - bin_low + 1) as f32).sqrt()
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// AudioReactiveSystem — core analysis state
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Audio analysis state — updated every frame from the music output.
#[derive(Debug, Clone)]
pub struct AudioReactiveSystem {
    /// Current FFT magnitude bins (N/2+1 values).
    pub fft_data: Vec<f32>,
    /// Smoothed bass energy (20-200 Hz), range [0, ~1].
    pub bass_energy: f32,
    /// Smoothed mid energy (200-2000 Hz), range [0, ~1].
    pub mid_energy: f32,
    /// Smoothed high energy (2000-20000 Hz), range [0, ~1].
    pub high_energy: f32,
    /// Whether a beat was detected this frame.
    pub beat_detected: bool,
    /// Smoothed amplitude envelope [0, 1].
    pub envelope: f32,

    // ── Internal state ──
    /// Previous bass energy for beat detection (onset).
    prev_bass: f32,
    /// Exponential moving average of bass for adaptive threshold.
    bass_ema: f32,
    /// Cooldown timer preventing rapid-fire beat triggers.
    beat_cooldown: f32,
    /// Raw (unsmoothed) energies for reactivity.
    raw_bass: f32,
    raw_mid: f32,
    raw_high: f32,
    /// Peak tracker for normalization.
    peak_bass: f32,
    peak_mid: f32,
    peak_high: f32,
    /// Accumulated time.
    time: f32,
}

impl AudioReactiveSystem {
    pub fn new() -> Self {
        Self {
            fft_data: vec![0.0; FFT_SIZE / 2 + 1],
            bass_energy: 0.0,
            mid_energy: 0.0,
            high_energy: 0.0,
            beat_detected: false,
            envelope: 0.0,
            prev_bass: 0.0,
            bass_ema: 0.0,
            beat_cooldown: 0.0,
            raw_bass: 0.0,
            raw_mid: 0.0,
            raw_high: 0.0,
            peak_bass: 0.01,
            peak_mid: 0.01,
            peak_high: 0.01,
            time: 0.0,
        }
    }

    /// Feed raw audio samples from the music engine output.
    /// Should be called once per frame with the latest audio buffer.
    pub fn process(&mut self, samples: &[f32], dt: f32) {
        self.time += dt;
        self.beat_cooldown = (self.beat_cooldown - dt).max(0.0);

        // Compute FFT magnitude spectrum
        self.fft_data = magnitude_spectrum(samples);

        // Extract raw band energies
        self.raw_bass = band_energy(&self.fft_data, BASS_LOW, BASS_HIGH);
        self.raw_mid = band_energy(&self.fft_data, MID_LOW, MID_HIGH);
        self.raw_high = band_energy(&self.fft_data, HIGH_LOW, HIGH_HIGH);

        // Adaptive peak tracking (slowly decay peaks for normalization)
        self.peak_bass = (self.peak_bass * 0.999).max(self.raw_bass).max(0.01);
        self.peak_mid = (self.peak_mid * 0.999).max(self.raw_mid).max(0.01);
        self.peak_high = (self.peak_high * 0.999).max(self.raw_high).max(0.01);

        // Normalize to [0, ~1] range
        let norm_bass = (self.raw_bass / self.peak_bass).min(1.0);
        let norm_mid = (self.raw_mid / self.peak_mid).min(1.0);
        let norm_high = (self.raw_high / self.peak_high).min(1.0);

        // Smooth with EMA
        self.bass_energy += (norm_bass - self.bass_energy) * BASS_SMOOTH;
        self.mid_energy += (norm_mid - self.mid_energy) * MID_SMOOTH;
        self.high_energy += (norm_high - self.high_energy) * HIGH_SMOOTH;

        // Envelope: RMS of the signal
        let rms = if !samples.is_empty() {
            (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
        } else {
            0.0
        };
        self.envelope += (rms.min(1.0) - self.envelope) * ENVELOPE_SMOOTH;

        // Beat detection: onset in bass band
        self.bass_ema += (self.raw_bass - self.bass_ema) * 0.1;
        self.beat_detected = self.raw_bass > self.bass_ema * BEAT_THRESHOLD
            && self.raw_bass > self.prev_bass
            && self.beat_cooldown <= 0.0
            && self.raw_bass > 0.01;

        if self.beat_detected {
            self.beat_cooldown = BEAT_COOLDOWN;
        }
        self.prev_bass = self.raw_bass;
    }

    /// Feed silence (no audio available — decay to zero).
    pub fn process_silence(&mut self, dt: f32) {
        self.time += dt;
        self.beat_cooldown = (self.beat_cooldown - dt).max(0.0);
        self.bass_energy *= 0.95;
        self.mid_energy *= 0.95;
        self.high_energy *= 0.95;
        self.envelope *= 0.95;
        self.beat_detected = false;
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// AudioReactiveOutput — what the renderer reads each frame
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Concrete visual modulations derived from audio analysis.
/// Read by the renderer every frame.
#[derive(Debug, Clone)]
pub struct AudioReactiveOutput {
    /// Multiplier for chaos-field particle speed (1.0 = normal).
    pub particle_speed_mult: f32,
    /// Camera FOV offset in degrees (negative = zoom in for punch).
    pub camera_fov_offset: f32,
    /// Ambient force field strength multiplier.
    pub force_field_strength: f32,
    /// Entity emission pulse intensity [0, 1].
    pub entity_emission_pulse: f32,
    /// Vignette intensity [0, 1] (louder → less vignette).
    pub vignette_intensity: f32,
    /// Whether a beat was detected this frame (for one-shot VFX).
    pub beat_detected: bool,
    /// Screen border flash color (RGBA, A=0 means no flash).
    pub border_flash: Vec4,
    /// Raw bass energy for external consumers.
    pub bass_energy: f32,
    /// Raw mid energy.
    pub mid_energy: f32,
    /// Raw high energy.
    pub high_energy: f32,
    /// Smoothed amplitude envelope.
    pub envelope: f32,
}

impl Default for AudioReactiveOutput {
    fn default() -> Self {
        Self {
            particle_speed_mult: 1.0,
            camera_fov_offset: 0.0,
            force_field_strength: 0.0,
            entity_emission_pulse: 0.0,
            vignette_intensity: VIGNETTE_BASE,
            beat_detected: false,
            border_flash: Vec4::ZERO,
            bass_energy: 0.0,
            mid_energy: 0.0,
            high_energy: 0.0,
            envelope: 0.0,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// AudioReactiveMapper — converts analysis to visual modulations
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Maps `AudioReactiveSystem` state to `AudioReactiveOutput` visual parameters.
/// Owns the FOV pulse decay state and border flash timer.
pub struct AudioReactiveMapper {
    /// Current FOV offset (decays back to 0).
    fov_offset: f32,
    /// Border flash remaining time.
    flash_timer: f32,
    /// Theme color for border flash (set externally).
    pub theme_color: Vec4,
    /// Global reactivity scale (0 = disabled, 1 = full).
    pub reactivity: f32,
    /// Whether the system is enabled.
    pub enabled: bool,
}

impl AudioReactiveMapper {
    pub fn new() -> Self {
        Self {
            fov_offset: 0.0,
            flash_timer: 0.0,
            theme_color: Vec4::new(0.4, 0.6, 1.0, 1.0), // default blue
            reactivity: 1.0,
            enabled: true,
        }
    }

    /// Set the theme color used for beat-triggered border flashes.
    pub fn set_theme_color(&mut self, color: Vec4) {
        self.theme_color = color;
    }

    /// Compute visual output from the analysis state.
    pub fn map(&mut self, audio: &AudioReactiveSystem, dt: f32) -> AudioReactiveOutput {
        if !self.enabled {
            return AudioReactiveOutput::default();
        }

        let r = self.reactivity;

        // Bass → particle speed
        let particle_speed_mult = 1.0 + audio.bass_energy * PARTICLE_SPEED_BASS_SCALE * r;

        // Beat → FOV pulse
        if audio.beat_detected {
            self.fov_offset = FOV_BEAT_PULSE * r;
            self.flash_timer = BORDER_FLASH_DURATION;
        }
        // Decay FOV back to 0
        if self.fov_offset < 0.0 {
            self.fov_offset += FOV_DECAY_RATE * dt;
            self.fov_offset = self.fov_offset.min(0.0);
        } else if self.fov_offset > 0.0 {
            self.fov_offset -= FOV_DECAY_RATE * dt;
            self.fov_offset = self.fov_offset.max(0.0);
        }

        // Mid → force field strength
        let force_field_strength = audio.mid_energy * FORCE_FIELD_MID_SCALE * r;

        // High → entity emission pulse
        let entity_emission_pulse = audio.high_energy * EMISSION_HIGH_SCALE * r;

        // Envelope → vignette (louder = less vignette)
        let vignette_intensity = VIGNETTE_BASE - audio.envelope * VIGNETTE_LOUD_REDUCTION * r;
        let vignette_intensity = vignette_intensity.clamp(0.1, 0.8);

        // Border flash
        self.flash_timer = (self.flash_timer - dt).max(0.0);
        let border_flash = if self.flash_timer > 0.0 {
            Vec4::new(
                self.theme_color.x,
                self.theme_color.y,
                self.theme_color.z,
                BORDER_FLASH_OPACITY * r,
            )
        } else {
            Vec4::ZERO
        };

        AudioReactiveOutput {
            particle_speed_mult,
            camera_fov_offset: self.fov_offset,
            force_field_strength,
            entity_emission_pulse,
            vignette_intensity,
            beat_detected: audio.beat_detected,
            border_flash,
            bass_energy: audio.bass_energy,
            mid_energy: audio.mid_energy,
            high_energy: audio.high_energy,
            envelope: audio.envelope,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Game integration — wires audio reactive to all visual systems
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Full audio-reactive pipeline: owns the analyzer + mapper, provides a
/// single `tick()` → `output()` interface for the game loop.
pub struct AudioReactivePipeline {
    pub analyzer: AudioReactiveSystem,
    pub mapper: AudioReactiveMapper,
    /// Cached output from the last tick.
    output: AudioReactiveOutput,
    /// Ring buffer for audio samples (accumulates between ticks).
    sample_buffer: Vec<f32>,
    /// Write cursor into the ring buffer.
    write_pos: usize,
}

impl AudioReactivePipeline {
    pub fn new() -> Self {
        Self {
            analyzer: AudioReactiveSystem::new(),
            mapper: AudioReactiveMapper::new(),
            output: AudioReactiveOutput::default(),
            sample_buffer: vec![0.0; FFT_SIZE],
            write_pos: 0,
        }
    }

    /// Push audio samples from the music engine into the ring buffer.
    /// Call this whenever new audio data is available.
    pub fn push_samples(&mut self, samples: &[f32]) {
        for &s in samples {
            self.sample_buffer[self.write_pos] = s;
            self.write_pos = (self.write_pos + 1) % FFT_SIZE;
        }
    }

    /// Tick the analysis pipeline (call once per frame).
    pub fn tick(&mut self, dt: f32) {
        // Reorder ring buffer into a contiguous slice for FFT
        let mut ordered = vec![0.0f32; FFT_SIZE];
        for i in 0..FFT_SIZE {
            ordered[i] = self.sample_buffer[(self.write_pos + i) % FFT_SIZE];
        }

        // Check if there's any actual audio
        let has_audio = ordered.iter().any(|s| s.abs() > 1e-6);
        if has_audio {
            self.analyzer.process(&ordered, dt);
        } else {
            self.analyzer.process_silence(dt);
        }

        self.output = self.mapper.map(&self.analyzer, dt);
    }

    /// Get the current visual output (read by the renderer).
    pub fn output(&self) -> &AudioReactiveOutput {
        &self.output
    }

    /// Convenience: set reactivity (0 = disabled, 1 = full).
    pub fn set_reactivity(&mut self, r: f32) {
        self.mapper.reactivity = r.clamp(0.0, 1.0);
    }

    /// Convenience: enable/disable the entire system.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.mapper.enabled = enabled;
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Visual applicator — applies AudioReactiveOutput to game systems
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Applies the audio-reactive output to various game visual parameters.
/// Call once per frame after `AudioReactivePipeline::tick()`.
pub struct VisualApplicator;

impl VisualApplicator {
    /// Apply particle speed modulation to chaos field parameters.
    pub fn apply_particle_speed(base_speed: f32, output: &AudioReactiveOutput) -> f32 {
        base_speed * output.particle_speed_mult
    }

    /// Apply camera FOV offset.
    pub fn apply_camera_fov(base_fov: f32, output: &AudioReactiveOutput) -> f32 {
        base_fov + output.camera_fov_offset
    }

    /// Apply force field strength modulation.
    pub fn apply_force_field_strength(base_strength: f32, output: &AudioReactiveOutput) -> f32 {
        base_strength * (1.0 + output.force_field_strength)
    }

    /// Apply entity emission pulse.
    pub fn apply_entity_emission(base_emission: f32, output: &AudioReactiveOutput) -> f32 {
        base_emission + output.entity_emission_pulse
    }

    /// Apply vignette intensity.
    pub fn apply_vignette(output: &AudioReactiveOutput) -> f32 {
        output.vignette_intensity
    }

    /// Get border flash color (Vec4::ZERO = no flash).
    pub fn border_flash(output: &AudioReactiveOutput) -> Vec4 {
        output.border_flash
    }

    /// Build a complete set of visual modifiers for the music bridge's
    /// `AudioVisualEffects` format.
    pub fn to_audio_visual_effects(output: &AudioReactiveOutput) -> AudioVisualEffectsCompat {
        AudioVisualEffectsCompat {
            particle_speed_mult: output.particle_speed_mult,
            camera_fov_offset: output.camera_fov_offset,
            force_field_strength: output.force_field_strength,
            entity_emission_pulse: output.entity_emission_pulse,
            vignette_intensity: output.vignette_intensity,
            beat_detected: output.beat_detected,
            bass_energy: output.bass_energy,
            high_energy: output.high_energy,
        }
    }
}

/// Compatible struct that mirrors `music_bridge::AudioVisualEffects`
/// for easy integration with existing code.
#[derive(Debug, Clone)]
pub struct AudioVisualEffectsCompat {
    pub particle_speed_mult: f32,
    pub camera_fov_offset: f32,
    pub force_field_strength: f32,
    pub entity_emission_pulse: f32,
    pub vignette_intensity: f32,
    pub beat_detected: bool,
    pub bass_energy: f32,
    pub high_energy: f32,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Spectral visualizer — for debug/UI display of the spectrum
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Reduces the FFT data to a smaller number of display bars for UI.
pub struct SpectrumVisualizer;

impl SpectrumVisualizer {
    /// Reduce FFT bins to `num_bars` display bars (logarithmic frequency scale).
    pub fn to_bars(fft_data: &[f32], num_bars: usize) -> Vec<f32> {
        if fft_data.is_empty() || num_bars == 0 {
            return vec![0.0; num_bars];
        }

        let n = fft_data.len();
        let mut bars = Vec::with_capacity(num_bars);

        for i in 0..num_bars {
            // Logarithmic frequency mapping
            let t0 = i as f32 / num_bars as f32;
            let t1 = (i + 1) as f32 / num_bars as f32;
            let bin0 = (n as f32 * (2.0_f32.powf(t0 * 10.0) - 1.0) / 1023.0) as usize;
            let bin1 = (n as f32 * (2.0_f32.powf(t1 * 10.0) - 1.0) / 1023.0) as usize;
            let bin0 = bin0.clamp(0, n - 1);
            let bin1 = bin1.clamp(bin0 + 1, n);

            let sum: f32 = fft_data[bin0..bin1].iter().sum();
            let avg = sum / (bin1 - bin0).max(1) as f32;
            bars.push(avg);
        }

        // Normalize to [0, 1]
        let max = bars.iter().cloned().fold(0.01_f32, f32::max);
        for b in &mut bars {
            *b /= max;
        }

        bars
    }

    /// Convert bars to glyph characters for text-mode spectrum display.
    pub fn bars_to_chars(bars: &[f32]) -> Vec<char> {
        bars.iter()
            .map(|&b| {
                if b < 0.1 { ' ' }
                else if b < 0.25 { '▁' }
                else if b < 0.4 { '▂' }
                else if b < 0.55 { '▃' }
                else if b < 0.7 { '▅' }
                else if b < 0.85 { '▆' }
                else { '█' }
            })
            .collect()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Tests
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[cfg(test)]
mod tests {
    use super::*;

    /// Generate a sine wave at a given frequency.
    fn sine_wave(freq: f32, samples: usize) -> Vec<f32> {
        (0..samples)
            .map(|i| (2.0 * PI * freq * i as f32 / SAMPLE_RATE).sin())
            .collect()
    }

    #[test]
    fn test_fft_sine_peak() {
        let signal = sine_wave(440.0, FFT_SIZE);
        let mag = magnitude_spectrum(&signal);
        // Peak should be near bin 440/HZ_PER_BIN
        let expected_bin = (440.0 / HZ_PER_BIN).round() as usize;
        let peak_bin = mag.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
        assert!((peak_bin as i32 - expected_bin as i32).abs() <= 2,
            "FFT peak at bin {peak_bin}, expected ~{expected_bin}");
    }

    #[test]
    fn test_band_energy_bass() {
        // 100 Hz sine should have energy in bass band
        let signal = sine_wave(100.0, FFT_SIZE);
        let mag = magnitude_spectrum(&signal);
        let bass = band_energy(&mag, BASS_LOW, BASS_HIGH);
        let high = band_energy(&mag, HIGH_LOW, HIGH_HIGH);
        assert!(bass > high * 5.0, "100Hz should be bass-heavy: bass={bass}, high={high}");
    }

    #[test]
    fn test_band_energy_high() {
        // 5000 Hz sine should have energy in high band
        let signal = sine_wave(5000.0, FFT_SIZE);
        let mag = magnitude_spectrum(&signal);
        let high = band_energy(&mag, HIGH_LOW, HIGH_HIGH);
        let bass = band_energy(&mag, BASS_LOW, BASS_HIGH);
        assert!(high > bass * 5.0, "5kHz should be high-heavy: high={high}, bass={bass}");
    }

    #[test]
    fn test_analyzer_silence() {
        let mut sys = AudioReactiveSystem::new();
        let silence = vec![0.0; FFT_SIZE];
        sys.process(&silence, 1.0 / 60.0);
        assert!(sys.bass_energy < 0.01);
        assert!(sys.mid_energy < 0.01);
        assert!(!sys.beat_detected);
    }

    #[test]
    fn test_analyzer_loud_signal() {
        let mut sys = AudioReactiveSystem::new();
        // Feed a loud bass signal
        let signal = sine_wave(80.0, FFT_SIZE);
        let loud: Vec<f32> = signal.iter().map(|s| s * 0.8).collect();
        for _ in 0..10 {
            sys.process(&loud, 1.0 / 60.0);
        }
        assert!(sys.bass_energy > 0.1, "bass should be significant: {}", sys.bass_energy);
        assert!(sys.envelope > 0.01, "envelope should be nonzero: {}", sys.envelope);
    }

    #[test]
    fn test_beat_detection() {
        let mut sys = AudioReactiveSystem::new();
        // Feed silence to establish baseline
        let silence = vec![0.0; FFT_SIZE];
        for _ in 0..30 {
            sys.process(&silence, 1.0 / 60.0);
        }
        // Sudden loud bass = beat
        let bass_hit: Vec<f32> = sine_wave(60.0, FFT_SIZE).iter().map(|s| s * 0.9).collect();
        sys.process(&bass_hit, 1.0 / 60.0);
        assert!(sys.beat_detected, "sudden bass should trigger beat");
    }

    #[test]
    fn test_beat_cooldown() {
        let mut sys = AudioReactiveSystem::new();
        let silence = vec![0.0; FFT_SIZE];
        for _ in 0..30 {
            sys.process(&silence, 1.0 / 60.0);
        }
        let bass: Vec<f32> = sine_wave(60.0, FFT_SIZE).iter().map(|s| s * 0.9).collect();
        sys.process(&bass, 1.0 / 60.0);
        assert!(sys.beat_detected);
        // Immediate second beat should be blocked by cooldown
        sys.process(&bass, 1.0 / 60.0);
        assert!(!sys.beat_detected, "cooldown should prevent rapid beats");
    }

    #[test]
    fn test_mapper_fov_pulse() {
        let mut mapper = AudioReactiveMapper::new();
        let mut audio = AudioReactiveSystem::new();
        audio.beat_detected = true;
        let out = mapper.map(&audio, 1.0 / 60.0);
        assert!(out.camera_fov_offset < 0.0, "beat should cause negative FOV offset");
    }

    #[test]
    fn test_mapper_particle_speed() {
        let mut mapper = AudioReactiveMapper::new();
        let mut audio = AudioReactiveSystem::new();
        audio.bass_energy = 0.8;
        let out = mapper.map(&audio, 1.0 / 60.0);
        assert!(out.particle_speed_mult > 1.0, "bass should increase particle speed");
    }

    #[test]
    fn test_mapper_vignette_reduction() {
        let mut mapper = AudioReactiveMapper::new();
        let mut audio = AudioReactiveSystem::new();
        audio.envelope = 0.8;
        let out = mapper.map(&audio, 1.0 / 60.0);
        assert!(out.vignette_intensity < VIGNETTE_BASE,
            "loud envelope should reduce vignette: {}", out.vignette_intensity);
    }

    #[test]
    fn test_mapper_border_flash() {
        let mut mapper = AudioReactiveMapper::new();
        let mut audio = AudioReactiveSystem::new();
        audio.beat_detected = true;
        let out = mapper.map(&audio, 1.0 / 60.0);
        assert!(out.border_flash.w > 0.0, "beat should trigger border flash");
    }

    #[test]
    fn test_pipeline_full() {
        let mut pipeline = AudioReactivePipeline::new();
        let signal = sine_wave(100.0, FFT_SIZE);
        pipeline.push_samples(&signal);
        pipeline.tick(1.0 / 60.0);
        let out = pipeline.output();
        // Should have some bass energy from 100Hz
        assert!(out.bass_energy >= 0.0);
        assert!(out.particle_speed_mult >= 1.0);
    }

    #[test]
    fn test_spectrum_visualizer() {
        let fft_data = vec![0.5; 100];
        let bars = SpectrumVisualizer::to_bars(&fft_data, 16);
        assert_eq!(bars.len(), 16);
        assert!(bars.iter().all(|b| *b >= 0.0 && *b <= 1.0));
    }

    #[test]
    fn test_bars_to_chars() {
        let bars = vec![0.0, 0.15, 0.3, 0.5, 0.65, 0.8, 0.95];
        let chars = SpectrumVisualizer::bars_to_chars(&bars);
        assert_eq!(chars.len(), 7);
        assert_eq!(chars[0], ' ');
        assert_eq!(chars[6], '█');
    }

    #[test]
    fn test_disabled_returns_default() {
        let mut mapper = AudioReactiveMapper::new();
        mapper.enabled = false;
        let mut audio = AudioReactiveSystem::new();
        audio.bass_energy = 1.0;
        audio.beat_detected = true;
        let out = mapper.map(&audio, 1.0 / 60.0);
        assert!((out.particle_speed_mult - 1.0).abs() < 0.01);
        assert!(!out.beat_detected);
    }

    #[test]
    fn test_process_silence_decays() {
        let mut sys = AudioReactiveSystem::new();
        sys.bass_energy = 0.5;
        sys.mid_energy = 0.5;
        for _ in 0..100 {
            sys.process_silence(1.0 / 60.0);
        }
        assert!(sys.bass_energy < 0.01);
        assert!(sys.mid_energy < 0.01);
    }
}
