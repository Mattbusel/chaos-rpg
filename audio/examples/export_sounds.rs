//! Write every synthesized sound effect to WAV files, without playing
//! anything. Handy for listening in an editor, or reusing them in a mod.
//!
//! Run with `cargo run -p chaos-rpg-audio --example export_sounds -- [DIR]`
//! (default `target/chaos-sounds`).

use chaos_rpg_audio::SoundBank;

fn main() -> std::io::Result<()> {
    let dir = std::path::PathBuf::from(
        std::env::args().nth(1).unwrap_or_else(|| "target/chaos-sounds".into()),
    );
    std::fs::create_dir_all(&dir)?;
    let bank = SoundBank::new();
    let mut files: Vec<(String, &Vec<u8>)> = vec![
        ("attack".into(), &bank.attack),
        ("heavy_attack".into(), &bank.heavy_attack),
        ("hit_normal".into(), &bank.hit_normal),
        ("hit_crit".into(), &bank.hit_crit),
        ("heal".into(), &bank.heal),
        ("death_player".into(), &bank.death_player),
        ("death_enemy".into(), &bank.death_enemy),
        ("level_up".into(), &bank.level_up),
        ("victory".into(), &bank.victory),
        ("game_over".into(), &bank.game_over),
        ("chaos_cascade".into(), &bank.chaos_cascade),
    ];
    for (i, w) in bank.engine_rolls.iter().enumerate() {
        files.push((format!("engine_roll_{i}"), w));
    }
    for (i, w) in bank.spells.iter().enumerate() {
        files.push((format!("spell_{i}"), w));
    }
    let mut bytes = 0;
    for (name, wav) in &files {
        std::fs::write(dir.join(format!("{name}.wav")), wav)?;
        bytes += wav.len();
    }
    println!("wrote {} WAV files ({} KB) to {}", files.len(), bytes / 1024, dir.display());
    Ok(())
}
