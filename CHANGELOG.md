# Changelog

## 2.3.0 (2026-10-04)

### Fixed

- Dates: four hand-written Unix-time-to-date conversions are replaced by the `time` crate. Two were wrong (2025-10-03 came out as 2025-10-20), so the Proof Engine frontend asked the leaderboard for the wrong day.
- Leaderboard: the reference server stores `seed` as a string, so every non-empty board failed to parse. Both forms are accepted now.
- Saves, scores and settings were written next to the executable, which fails silently when that folder is read-only and leaves files in `~/.cargo/bin` after `cargo install`. They now go to the per-user data folder (`directories` crate), and existing files next to the executable are still found.

### Added

- `chaos_rpg_core::dice::ChaosDie`: chaos dice as a `rand` distribution, so any `rand::Rng` can roll them (`rng.sample(ChaosDie::d20())`). They are deliberately not fair; the new `dice_stats` example measures how far off (a d20 shows 20 10.2% of the time).
- Examples: `quickstart` and `dice_stats` (core), `export_sounds` (audio, writes the game's sounds as WAV files).
- `core/benches/game.rs` (criterion). Against 2.2.1 on the same machine: chaos roll 1.43 -> 1.38 us, destiny roll 2.32 -> 2.25 us, floor generation 23.7 -> 22.9 us; within a few percent, nothing slower.
- `core/tests/wav_cross_check.rs`: WAV files written by the audio synth decode with `hound` to the same samples.
- Doc comments across the core library (more modules and public items documented on docs.rs); GitLab CI runs tests, clippy and docs.

### Changed

- The Proof Engine frontend uses `proof-engine` 0.3, which loads real image and sound files and makes real HTTP requests (0.2 returned placeholders).
- Clippy is clean with `-D warnings` across the workspace.

## 2.2.1 (2026-09-25)

- One-line installs: `install.ps1` (Windows), `install.sh` (macOS/Linux), Scoop, Homebrew and `cargo binstall`, all pulling the release archives and checking SHA-256.
- `--help` on every frontend now gives examples, the main keys and where the settings file lives; an unknown option says so and exits instead of starting the game.
- README rebuilt around install, three steps to play, and real output: a GIF captured from the Proof Engine frontend and the actual chaos pipeline trace. The pipeline section now matches the code.
- `chaos-rpg-core` docs open with a runnable example, and a new `roll` example prints a full chaos roll. Two broken doc examples fixed; CI now runs doc tests.
- Clearer crates.io descriptions and keywords.

## 2.2.0 (2026-09-25)

- Prebuilt downloads for Windows, macOS (Apple Silicon and Intel) and Linux on every release, with all three frontends in one archive and a `SHA256SUMS.txt`.
- `cargo install chaos-rpg`, `cargo install chaos-rpg-graphical` and `cargo install chaos-rpg-proof` now work.
- A fresh clone builds on its own: the Proof Engine frontend uses `proof-engine` 0.2 from crates.io instead of a sibling checkout.
- Fixed borrow errors in the Proof Engine frontend (fluid arena, soft-body death burst) that stopped it compiling.
- Every frontend answers `--help` and `--version`.
- CI builds all native frontends on Linux, macOS and Windows again.
- Release binaries are no longer committed to `dist/`; old ones stay in git history.
- Workspace version now matches the release tags.
