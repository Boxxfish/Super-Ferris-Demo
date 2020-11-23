///
/// Component that represents a position.
/// 

use super::Component;

pub struct PositionComponent {
    pub exists: bool,
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub prec_x: f32,
    pub prec_y: f32,
    pub spd_x: f32,
    pub spd_y: f32
}

impl Component for PositionComponent {
    fn uninit() -> Self {
        Self {
            exists: false,
            id: 0,
            x: 0,
            y: 0,
            prec_x: 0.0,
            prec_y: 0.0,
            spd_x: 0.0,
            spd_y: 0.0
        }
    }
}