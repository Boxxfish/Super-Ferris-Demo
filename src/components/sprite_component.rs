///
/// Component that represents a sprite to be drawn.
/// 

use super::Component;

pub struct SpriteComponent {
    pub exists: bool,
    pub id: u32,
    pub quad_id: u32,
    pub tex_name: String,
    pub sprite_index: u32,
    pub tilemap: Option<Vec<u32>>,
    pub tilemap_width: u32,
    pub tilemap_height: u32,
    pub should_update: bool,
}

impl Component for SpriteComponent {
    fn uninit() -> Self {
        Self {
            exists: false,
            id: 0,
            quad_id: 0,
            tex_name: String::from("black"),
            sprite_index: 0,
            tilemap: None,
            tilemap_width: 0,
            tilemap_height: 0,
            should_update: true
        }
    }
}