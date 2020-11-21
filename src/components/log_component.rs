///
/// A component that logs info.
/// 

use super::Component;

pub struct LogComponent {
    pub exists: bool,
    pub id: u32,
    pub has_info: bool,
    pub message: String
}

impl Component for LogComponent {
    fn uninit() -> Self {
        Self {
            exists: false,
            id: 0,
            has_info: false,
            message: String::from("")
        }
    }
}