//! Обработка изображений для извлечения параметров термокарста

pub mod photo;
pub mod detection;
pub mod downloader;
pub mod satellite;

pub use photo::*;
pub use detection::*;
pub use downloader::*;
pub use satellite::*;
