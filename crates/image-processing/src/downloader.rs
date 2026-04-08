//! Загрузка примеров изображений термокарста

use anyhow::Result;
use std::path::Path;

/// Загрузчик примеров изображений
pub struct ImageDownloader;

impl ImageDownloader {
    #[cfg(feature = "download")]
    /// Загрузить пример изображения термокарста
    pub fn download_example(output_path: &Path) -> Result<()> {
        println!("📥 Загрузка примера изображения термокарста...");

        // Используем публичное изображение термокарста из Wikimedia Commons
        // Это реальная фотография термокарстового озера в Арктике
        let url = "https://upload.wikimedia.org/wikipedia/commons/thumb/8/8f/Thermokarst_lake.jpg/800px-Thermokarst_lake.jpg";

        let response = reqwest::blocking::get(url)?;
        let bytes = response.bytes()?;

        std::fs::write(output_path, bytes)?;

        println!("✅ Изображение загружено: {:?}", output_path);
        Ok(())
    }

    #[cfg(feature = "download")]
    /// Загрузить несколько примеров
    pub fn download_examples(output_dir: &Path) -> Result<Vec<String>> {
        std::fs::create_dir_all(output_dir)?;

        let examples = vec![
            (
                "thermokarst_lake_1.jpg",
                "https://upload.wikimedia.org/wikipedia/commons/thumb/8/8f/Thermokarst_lake.jpg/800px-Thermokarst_lake.jpg"
            ),
            (
                "thermokarst_lake_2.jpg",
                "https://upload.wikimedia.org/wikipedia/commons/thumb/3/3e/Thermokarst_lakes_Lena_Delta.jpg/800px-Thermokarst_lakes_Lena_Delta.jpg"
            ),
        ];

        let mut downloaded = Vec::new();

        for (filename, url) in examples {
            let path = output_dir.join(filename);

            println!("📥 Загрузка {}...", filename);

            match reqwest::blocking::get(url) {
                Ok(response) => {
                    if let Ok(bytes) = response.bytes() {
                        if std::fs::write(&path, bytes).is_ok() {
                            println!("✅ Загружено: {}", filename);
                            downloaded.push(filename.to_string());
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️  Ошибка загрузки {}: {}", filename, e);
                }
            }
        }

        Ok(downloaded)
    }

    /// Создать синтетическое изображение для тестирования
    pub fn create_synthetic_thermokarst(
        output_path: &Path,
        diameter_pixels: u32,
    ) -> Result<()> {
        use image::{ImageBuffer, Rgb, RgbImage};

        let size = diameter_pixels * 2;
        let mut img: RgbImage = ImageBuffer::new(size, size);

        let center = (size / 2, size / 2);
        let radius = diameter_pixels / 2;

        // Создать круглое термокарстовое образование
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let dx = x as i32 - center.0 as i32;
            let dy = y as i32 - center.1 as i32;
            let distance = ((dx * dx + dy * dy) as f32).sqrt();

            if distance < radius as f32 {
                // Внутри - темнее (вода/просадка)
                *pixel = Rgb([50, 70, 90]);
            } else if distance < (radius + 10) as f32 {
                // Край - переходная зона
                *pixel = Rgb([100, 120, 100]);
            } else {
                // Снаружи - тундра
                *pixel = Rgb([150, 160, 140]);
            }
        }

        img.save(output_path)?;
        println!("✅ Создано синтетическое изображение: {:?}", output_path);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_synthetic() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("synthetic.jpg");

        ImageDownloader::create_synthetic_thermokarst(&path, 200).unwrap();
        assert!(path.exists());
    }
}
