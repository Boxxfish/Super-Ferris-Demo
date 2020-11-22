///
/// Component that represents a position.
/// 

use super::Component;

pub struct PositionComponent {
    pub exists: bool,
    pub id: u32,
    pub x: i32,
    pub y: i32
}

impl Component for PositionComponent {
    fn uninit() -> Self {
        Self {
            exists: false,
            id: 0,
            x: 0,
            y: 0
        }
    }
}