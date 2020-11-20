///
/// Trait for components.
/// 

pub trait Component {
    /// Returns an uninitialized instance of this component.
    fn uninit() -> Self;
}