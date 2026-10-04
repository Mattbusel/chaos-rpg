# Contributing to CHAOS RPG

Issues and merge requests are welcome at https://gitlab.com/mattbusel/chaos-rpg. For a large change, open an issue first.

## Build and test

```bash
git clone https://gitlab.com/mattbusel/chaos-rpg
cd chaos-rpg
cargo test --workspace          # game logic tests; nothing opens a window
cargo run --release -p chaos-rpg-graphical
```

Linux needs the ALSA, X11 and GL headers for the frontends:
`sudo apt install pkg-config libasound2-dev libx11-dev libxcursor-dev libxrandr-dev libxi-dev libxkbcommon-dev libgl1-mesa-dev libwayland-dev`.

## Where things live

- `core/`: all game rules and the chaos pipeline (`chaos_pipeline.rs`, `math_engines.rs`). Changes here affect every frontend; add a unit test.
- `graphical/`, `graphical-proof/`, `terminal/`: the three frontends.
- `audio/`: procedural music and sound.

The chaos pipeline parameters (Lorenz sigma/rho/beta, Mandelbrot iterations, logistic r range) are tuned on purpose: changing them changes every roll and every seeded run. Run `cargo run -p chaos-rpg-core --example dice_stats` before and after and say what moved.

## Before you send a change

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
