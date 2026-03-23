// CP437 sprite definitions for enemies, player classes, and bosses.
// Each sprite is a slice of (dx, dy, cp437_glyph, fg_rgb, bg_rgb) tuples.

pub type GlyphTuple = (i32, i32, u16, (u8, u8, u8), (u8, u8, u8));

/// A sprite is a list of individually-colored CP437 glyphs at relative offsets.
pub struct Sprite {
    pub glyphs: &'static [GlyphTuple],
    pub width: i32,
    pub height: i32,
}

// ── Player class sprites ─────────────────────────────────────────────────────

pub static SPRITE_MAGE: Sprite = Sprite {
    width: 3,
    height: 4,
    glyphs: &[
        (1, 0, b'^' as u16, (200, 100, 255), (0, 0, 0)),
        (0, 1, b'/' as u16, (150, 80, 200), (0, 0, 0)),
        (1, 1, b'@' as u16, (220, 180, 255), (0, 0, 0)),
        (2, 1, b'\\' as u16, (150, 80, 200), (0, 0, 0)),
        (0, 2, b'|' as u16, (100, 60, 150), (0, 0, 0)),
        (1, 2, b'|' as u16, (100, 60, 150), (0, 0, 0)),
        (2, 2, b'*' as u16, (200, 100, 255), (0, 0, 0)),
        (0, 3, b'\\' as u16, (80, 50, 120), (0, 0, 0)),
        (2, 3, b'/' as u16, (80, 50, 120), (0, 0, 0)),
    ],
};

pub static SPRITE_BERSERKER: Sprite = Sprite {
    width: 3,
    height: 4,
    glyphs: &[
        (1, 0, b'O' as u16, (200, 80, 80), (0, 0, 0)),
        (0, 1, b'[' as u16, (180, 60, 60), (0, 0, 0)),
        (1, 1, b'@' as u16, (220, 100, 100), (0, 0, 0)),
        (2, 1, b']' as u16, (180, 60, 60), (0, 0, 0)),
        (0, 2, b'/' as u16, (200, 80, 80), (0, 0, 0)),
        (1, 2, b'H' as u16, (160, 60, 60), (0, 0, 0)),
        (2, 2, b'\\' as u16, (200, 80, 80), (0, 0, 0)),
        (0, 3, b'/' as u16, (140, 50, 50), (0, 0, 0)),
        (2, 3, b'\\' as u16, (140, 50, 50), (0, 0, 0)),
    ],
};

// ── Enemy sprites ────────────────────────────────────────────────────────────

pub static SPRITE_FRACTAL_IMP: Sprite = Sprite {
    width: 3,
    height: 3,
    glyphs: &[
        (1, 0, b'v' as u16, (200, 50, 200), (0, 0, 0)),
        (0, 1, b'(' as u16, (150, 30, 150), (0, 0, 0)),
        (1, 1, b'i' as u16, (220, 80, 220), (0, 0, 0)),
        (2, 1, b')' as u16, (150, 30, 150), (0, 0, 0)),
        (0, 2, b'/' as u16, (100, 20, 100), (0, 0, 0)),
        (2, 2, b'\\' as u16, (100, 20, 100), (0, 0, 0)),
    ],
};

pub static SPRITE_ENTROPY_SPRITE: Sprite = Sprite {
    width: 3,
    height: 3,
    glyphs: &[
        (1, 0, b'*' as u16, (255, 200, 50), (0, 0, 0)),
        (0, 1, b'~' as u16, (200, 150, 30), (0, 0, 0)),
        (1, 1, b'&' as u16, (255, 220, 80), (0, 0, 0)),
        (2, 1, b'~' as u16, (200, 150, 30), (0, 0, 0)),
        (0, 2, b'\\' as u16, (180, 130, 20), (0, 0, 0)),
        (2, 2, b'/' as u16, (180, 130, 20), (0, 0, 0)),
    ],
};

pub static SPRITE_BOSS_GENERIC: Sprite = Sprite {
    width: 5,
    height: 5,
    glyphs: &[
        (2, 0, b'V' as u16, (255, 50, 50), (0, 0, 0)),
        (0, 1, b'[' as u16, (200, 30, 30), (0, 0, 0)),
        (1, 1, b'=' as u16, (180, 20, 20), (0, 0, 0)),
        (2, 1, b'@' as u16, (255, 80, 80), (0, 0, 0)),
        (3, 1, b'=' as u16, (180, 20, 20), (0, 0, 0)),
        (4, 1, b']' as u16, (200, 30, 30), (0, 0, 0)),
        (0, 2, b'|' as u16, (200, 30, 30), (0, 0, 0)),
        (1, 2, b'{' as u16, (220, 50, 50), (0, 0, 0)),
        (2, 2, b'X' as u16, (255, 100, 50), (0, 0, 0)),
        (3, 2, b'}' as u16, (220, 50, 50), (0, 0, 0)),
        (4, 2, b'|' as u16, (200, 30, 30), (0, 0, 0)),
        (0, 3, b'/' as u16, (180, 20, 20), (0, 0, 0)),
        (2, 3, b'W' as u16, (200, 30, 30), (0, 0, 0)),
        (4, 3, b'\\' as u16, (180, 20, 20), (0, 0, 0)),
        (0, 4, b'/' as u16, (150, 10, 10), (0, 0, 0)),
        (4, 4, b'\\' as u16, (150, 10, 10), (0, 0, 0)),
    ],
};

/// Draw a sprite at screen position (x, y).
pub fn draw_sprite(ctx: &mut bracket_lib::prelude::BTerm, sprite: &Sprite, x: i32, y: i32) {
    use bracket_lib::prelude::*;
    for &(dx, dy, glyph, fg, _bg) in sprite.glyphs {
        ctx.set(x + dx, y + dy, RGB::named(fg), RGB::named(BLACK), glyph);
    }
}
