// Pre-generated sound bank — all sounds synthesized at startup.
// Each entry is a WAV byte buffer ready to hand to rodio.

use chaos_rpg_core::audio_events::AudioEvent;
use chaos_rpg_core::audio_synth as synth;
use rodio::{OutputStreamHandle, Decoder, Sink};
use std::io::Cursor;

pub struct SoundBank {
    pub attack:            Vec<u8>,
    pub heavy_attack:      Vec<u8>,
    pub hit_normal:        Vec<u8>,
    pub hit_crit:          Vec<u8>,
    pub heal:              Vec<u8>,
    pub death_player:      Vec<u8>,
    pub death_enemy:       Vec<u8>,
    pub level_up:          Vec<u8>,
    pub menu_nav:          Vec<u8>,
    pub menu_confirm:      Vec<u8>,
    pub menu_cancel:       Vec<u8>,
    pub item_pickup:       Vec<u8>,
    pub shop_enter:        Vec<u8>,
    pub trap_hit:          Vec<u8>,
    pub trap_disarmed:     Vec<u8>,
    pub floor_transition:  Vec<u8>,
    pub victory:           Vec<u8>,
    pub game_over:         Vec<u8>,
    pub hunger:            Vec<u8>,
    pub boss_tier1:        Vec<u8>,
    pub boss_tier2:        Vec<u8>,
    pub boss_tier3:        Vec<u8>,
    pub nemesis:           Vec<u8>,
    pub boon:              Vec<u8>,
    pub volatility_reroll: Vec<u8>,
    pub chaos_cascade:     Vec<u8>,
    // Per-engine rolls: 10 entries
    pub engine_rolls:      [Vec<u8>; 10],
    // Per-spell SFX: 8 variants
    pub spells:            [Vec<u8>; 8],
    // Per-craft SFX: 6 ops × 2 (success/fail)
    pub craft_success:     [Vec<u8>; 6],
    pub craft_fail:        [Vec<u8>; 6],
}

impl SoundBank {
    pub fn new() -> Self {
        let engine_rolls = std::array::from_fn(|i| {
            synth::encode_wav(&synth::sfx_engine_roll(i as u8))
        });
        let spells = std::array::from_fn(|i| {
            synth::encode_wav(&synth::sfx_spell(i))
        });
        let craft_success = std::array::from_fn(|i| {
            synth::encode_wav(&synth::sfx_craft(i, true))
        });
        let craft_fail = std::array::from_fn(|i| {
            synth::encode_wav(&synth::sfx_craft(i, false))
        });

        Self {
            attack:            synth::encode_wav(&synth::sfx_attack()),
            heavy_attack:      synth::encode_wav(&synth::sfx_heavy_attack()),
            hit_normal:        synth::encode_wav(&synth::sfx_hit(false)),
            hit_crit:          synth::encode_wav(&synth::sfx_hit(true)),
            heal:              synth::encode_wav(&synth::sfx_heal()),
            death_player:      synth::encode_wav(&synth::sfx_death_player()),
            death_enemy:       synth::encode_wav(&synth::sfx_death_enemy()),
            level_up:          synth::encode_wav(&synth::sfx_level_up()),
            menu_nav:          synth::encode_wav(&synth::sfx_menu_navigate()),
            menu_confirm:      synth::encode_wav(&synth::sfx_menu_confirm()),
            menu_cancel:       synth::encode_wav(&synth::sfx_menu_cancel()),
            item_pickup:       synth::encode_wav(&synth::sfx_item_pickup()),
            shop_enter:        synth::encode_wav(&synth::sfx_shop_enter()),
            trap_hit:          synth::encode_wav(&synth::sfx_trap(false)),
            trap_disarmed:     synth::encode_wav(&synth::sfx_trap(true)),
            floor_transition:  synth::encode_wav(&synth::sfx_floor_transition()),
            victory:           synth::encode_wav(&synth::sfx_victory()),
            game_over:         synth::encode_wav(&synth::sfx_game_over()),
            hunger:            synth::encode_wav(&synth::sfx_hunger()),
            boss_tier1:        synth::encode_wav(&synth::sfx_boss_start(1)),
            boss_tier2:        synth::encode_wav(&synth::sfx_boss_start(2)),
            boss_tier3:        synth::encode_wav(&synth::sfx_boss_start(3)),
            nemesis:           synth::encode_wav(&synth::sfx_nemesis_spawned()),
            boon:              synth::encode_wav(&synth::sfx_boon_selected()),
            volatility_reroll: synth::encode_wav(&synth::sfx_volatility_reroll()),
            chaos_cascade:     synth::encode_wav(&synth::sfx_chaos_cascade(3)),
            engine_rolls,
            spells,
            craft_success,
            craft_fail,
        }
    }

    /// Play a WAV buffer in fire-and-forget fashion (detached sink).
    fn play_wav(handle: &OutputStreamHandle, wav: &[u8]) {
        let cursor = Cursor::new(wav.to_vec());
        if let Ok(source) = Decoder::new(cursor) {
            if let Ok(sink) = Sink::try_new(handle) {
                sink.append(source);
                sink.detach();
            }
        }
    }

    /// Route an AudioEvent to the appropriate SFX buffer and play it.
    pub fn play(&self, handle: &OutputStreamHandle, event: &AudioEvent) {
        use AudioEvent::*;
        let wav: Option<&[u8]> = match event {
            PlayerAttack                           => Some(&self.attack),
            PlayerHeavyAttack                      => Some(&self.heavy_attack),
            EnemyAttack                            => Some(&self.hit_normal),
            DamageDealt { is_crit: true, .. }      => Some(&self.hit_crit),
            DamageDealt { is_crit: false, .. }     => Some(&self.hit_normal),
            HealApplied { .. }                     => Some(&self.heal),
            PlayerDefend                           => Some(&self.menu_cancel),
            SpellCast { spell_index }              => self.spells.get(*spell_index % 8).map(|v| v.as_slice()),
            PlayerFled                             => Some(&self.menu_cancel),
            EntityDied { is_player: true }         => Some(&self.death_player),
            EntityDied { is_player: false }        => Some(&self.death_enemy),
            LevelUp                                => Some(&self.level_up),
            StatusApplied                          => Some(&self.menu_nav),
            BossEncounterStart { boss_tier }       => match boss_tier {
                1 => Some(&self.boss_tier1),
                2 => Some(&self.boss_tier2),
                _ => Some(&self.boss_tier3),
            },
            GauntletStart                          => Some(&self.boss_tier1),
            GauntletStageClear { .. }              => Some(&self.level_up),
            ChaosEngineRoll { engine_id }          => self.engine_rolls.get((*engine_id % 10) as usize).map(|v| v.as_slice()),
            DestinyRoll                            => Some(&self.engine_rolls[0]),
            EngineCritical                         => Some(&self.hit_crit),
            ChaosCascade { .. }                    => Some(&self.chaos_cascade),
            TrapTriggered { disarmed: true }       => Some(&self.trap_disarmed),
            TrapTriggered { disarmed: false }      => Some(&self.trap_hit),
            ShopEntered                            => Some(&self.shop_enter),
            ItemPurchased                          => Some(&self.item_pickup),
            BoonSelected                           => Some(&self.boon),
            RestTaken                              => Some(&self.heal),
            MysteryRoom                            => Some(&self.menu_nav),
            CursedFloorActivated                   => Some(&self.hit_crit),
            HungerTriggered                        => Some(&self.hunger),
            BloodPactDrain                         => Some(&self.hit_normal),
            ItemPickup                             => Some(&self.item_pickup),
            CraftStart { op_index }                => self.craft_success.get(*op_index % 6).map(|v| v.as_slice()),
            CraftSuccess                           => Some(&self.craft_success[0]),
            CraftFail                              => Some(&self.craft_fail[0]),
            ItemVolatilityReroll                   => Some(&self.volatility_reroll),
            SkillCheckResult { success: true }     => Some(&self.menu_confirm),
            SkillCheckResult { success: false }    => Some(&self.menu_cancel),
            MenuNavigate                           => Some(&self.menu_nav),
            MenuConfirm                            => Some(&self.menu_confirm),
            MenuCancel                             => Some(&self.menu_cancel),
            GameOver                               => Some(&self.game_over),
            Victory                                => Some(&self.victory),
            DailyStart                             => Some(&self.menu_confirm),
            NemesisSpawned                         => Some(&self.nemesis),
            FloorEntered { .. }                    => Some(&self.floor_transition),
            RoomEntered { .. }                     => None, // silent room transitions
        };
        if let Some(buf) = wav {
            Self::play_wav(handle, buf);
        }
    }
}
