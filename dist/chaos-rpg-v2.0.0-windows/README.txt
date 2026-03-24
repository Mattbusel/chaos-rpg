CHAOS RPG v2.0.0 — Where Math Goes To Die
==========================================

A roguelike where every outcome is determined by chaining 10 mathematical
algorithms. No dice. Pure chaos. Now powered by the Proof Engine — a custom
221,000+ line mathematical rendering engine built from scratch.

EXECUTABLES
-----------

  chaos-rpg-proof.exe     ← THE NEW FRONTEND (Proof Engine)
                            Full PBR lighting, shader graph, particle physics,
                            3D mathematical animations, 12 unique boss visuals,
                            procedural music, weather, terrain, and more.

  chaos-rpg-graphical.exe ← LEGACY FRONTEND (bracket-lib)
                            The original ASCII graphical frontend.
                            Included as a fallback.

HOW TO PLAY
-----------

  1. Double-click chaos-rpg-proof.exe
  2. Select a theme (press [T] on title screen)
  3. Choose New Run → pick class, background, difficulty
  4. Choose a starting boon
  5. Navigate rooms, fight enemies, find loot, descend deeper
  6. Try not to die (you will die)

CONTROLS
--------

  Title:     Up/Down = navigate, Enter = select, T = theme, Q = quit
  Combat:    A = attack, H = heavy, D = defend, F = flee, T = taunt
             1-8 = cast spells, V = chaos engine visualizer
  Floor Map: Up/Down = select room, E/Enter = enter, D = descend
             C = character sheet, N = passive tree
  General:   Esc = back, Tab = cycle tabs

WHAT'S NEW IN v2.0
-------------------

  - PROOF ENGINE: Custom 221K-line mathematical rendering engine
  - PBR LIGHTING: Per-room, per-boss, per-floor dynamic lighting
  - SHADER GRAPH: 5 theme presets, floor-depth visual degradation,
    corruption glitch effects, boss-specific shader overrides
  - 12 BOSS VISUALS: Mirror symmetry, Null progressive blackout,
    Algorithm Reborn 3-phase chaos takeover, and more
  - CINEMATICS: Death sequence (5-phase), victory celebration,
    boss entrances with typewriter reveal, level-up gold pillar
  - WEATHER: Digital rain, compute pulses, electrical storms,
    void snow — reactive to floor depth and boss fights
  - AI: Steering behaviors, behavior trees, GOAP planning,
    utility AI with logistic scoring curves
  - DIALOGUE: Archivist reputation-reactive greetings, boss combat
    dialogue, Mathematician Fragment codex trees
  - REPLAY: Automatic recording, playback with speed control,
    ghost runs for daily seed competition
  - ECONOMY: Supply/demand pricing, faction treasuries,
    reputation discounts, Archivist price announcements
  - TERRAIN: Isometric noise-based floor map, room-type elevation
  - MODDING: Script hooks, mod.toml manifests, hot-reload
  - 5 SAVE SLOTS: Visual state persistence, cloud sync ready
  - PASSIVE TREE: 820+ node browser with allocation
  - FULL TUTORIAL: 5 slides explaining chaos math

CONFIGURATION
-------------

  Edit chaos_config.toml for:
  - Audio settings (music vibe: Off/Minimal/Classic/Chill)
  - Leaderboard endpoint
  - Gameplay modifiers

CREDITS
-------

  Game: Matthew Busel
  Engine: Proof Engine (https://github.com/Mattbusel/proof-engine)
  Source: https://github.com/Mattbusel/chaos-rpg
