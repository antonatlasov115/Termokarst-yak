//! Геологические процессы и расчеты

pub mod consolidation;
pub mod lateral_expansion;
pub mod stability;

pub use consolidation::*;
pub use lateral_expansion::*;
pub use stability::*;
