//! `audio_synth::encode_wav` writes WAV files by hand; check them with the
//! `hound` crate, which is what most Rust audio code uses to read WAVs.

use chaos_rpg_core::audio_synth::{encode_wav, sfx_engine_roll, SAMPLE_RATE};

#[test]
fn encoded_wav_reads_back_with_hound() {
    let samples = sfx_engine_roll(3);
    assert!(!samples.is_empty());
    let wav = encode_wav(&samples);
    let mut reader = hound::WavReader::new(std::io::Cursor::new(wav)).expect("valid WAV header");
    let spec = reader.spec();
    assert_eq!(spec.channels, 1);
    assert_eq!(spec.sample_rate, SAMPLE_RATE);
    assert_eq!(spec.bits_per_sample, 16);
    assert_eq!(spec.sample_format, hound::SampleFormat::Int);
    let decoded: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
    assert_eq!(decoded.len(), samples.len());
    for (d, s) in decoded.iter().zip(&samples) {
        let want = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        assert_eq!(*d, want);
    }
}

#[test]
fn out_of_range_and_nan_samples_are_clamped() {
    let wav = encode_wav(&[2.0, -2.0, f32::NAN, 0.5]);
    let mut reader = hound::WavReader::new(std::io::Cursor::new(wav)).unwrap();
    let decoded: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
    assert_eq!(decoded, vec![32767, -32767, 0, 16383]);
}
