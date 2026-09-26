# Changelog

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
