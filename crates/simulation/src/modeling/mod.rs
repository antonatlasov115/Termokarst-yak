//! Моделирование термокарстовых процессов

pub mod engine;
pub mod inverse_modeling;
pub mod satellite_integration;

pub use engine::*;
pub use inverse_modeling::*;
pub use satellite_integration::*;
