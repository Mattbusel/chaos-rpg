//! Player entity — AmorphousEntity formation per class.

use proof_engine::prelude::*;
use chaos_rpg_core::character::CharacterClass;

/// Build an AmorphousEntity for the player based on their class.
pub fn build_player_entity(class: CharacterClass, position: Vec3) -> AmorphousEntity {
    let (chars, color) = class_visual(class);

    let mut entity = AmorphousEntity::default();
    entity.position = position;
    entity.entity_mass = 50.0;

    // Build formation: diamond pattern
    let size: i32 = 3;
    let mut positions = Vec::new();
    let mut formation_chars = Vec::new();
    let mut colors = Vec::new();

    for dy in -size..=size {
        let width = size - dy.abs();
        for dx in -width..=width {
            positions.push(Vec3::new(dx as f32 * 0.6, dy as f32 * 0.5, 0.0));
            let idx = ((dx + dy + size * 2) as usize) % chars.len();
            formation_chars.push(chars[idx]);
            colors.push(color);
        }
    }

    entity.formation = positions;
    entity.formation_chars = formation_chars;
    entity.formation_colors = colors;
    entity
}

fn class_visual(class: CharacterClass) -> (Vec<char>, Vec4) {
    match class {
        CharacterClass::Berserker => (
            vec!['>', '<', '!', '#', '█', '▓', 'X', '×'],
            Vec4::new(0.85, 0.2, 0.15, 1.0),
        ),
        CharacterClass::Mage => (
            vec!['*', '◆', '∞', '∂', '∑', '◇', '·', '○'],
            Vec4::new(0.4, 0.3, 0.95, 1.0),
        ),
        CharacterClass::Ranger => (
            vec!['/', '\\', '|', '>', '<', '·', '→', '←'],
            Vec4::new(0.3, 0.8, 0.2, 1.0),
        ),
        CharacterClass::Thief => (
            vec!['.', '·', '~', '-', '\'', '`', ':', ';'],
            Vec4::new(0.5, 0.5, 0.5, 1.0),
        ),
        CharacterClass::Necromancer => (
            vec!['☠', '†', '‡', '·', '○', '●', '∴', '∵'],
            Vec4::new(0.3, 0.7, 0.3, 1.0),
        ),
        CharacterClass::Alchemist => (
            vec!['~', '≈', '◇', '○', '●', '∆', '▽', '+'],
            Vec4::new(0.7, 0.5, 0.9, 1.0),
        ),
        CharacterClass::Paladin => (
            vec!['+', '†', '■', '▓', '█', '◆', '|', '─'],
            Vec4::new(0.9, 0.85, 0.4, 1.0),
        ),
        CharacterClass::VoidWalker => (
            vec![' ', '·', '.', '~', '░', ' ', '▒', ' '],
            Vec4::new(0.6, 0.2, 0.8, 1.0),
        ),
        _ => (
            vec!['◆', '◇', '■', '□', '●', '○', '▪', '▫'],
            Vec4::new(0.7, 0.7, 0.7, 1.0),
        ),
    }
}
