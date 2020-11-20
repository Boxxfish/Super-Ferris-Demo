///
/// A component that logs info.
/// 

use std::string;

pub struct LogComponent {
    pub exists: bool,
    pub id: u32,
    pub has_info: bool,
    pub message: String
}

impl LogComponent {
    /// Creates an uninitialized entity.
    pub fn uninit() -> Self {
        Self {
            exists: false,
            id: 0,
            has_info: false,
            message: String::from("")
        }
    }
}